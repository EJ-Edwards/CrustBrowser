// --- takeover.rs ---
// Subdomain-takeover prober. Given a hostname (typically a CNAME target), decide
// whether it points at a claimable / dangling third-party resource that an
// attacker could re-register and use to serve content on someone's subdomain.
//
// This is the "internal browser check" role CrustBrowser is well suited for: it
// fetches the target and matches the response against known "unclaimed resource"
// fingerprints (seeded from the community can-i-take-over-xyz catalogue).
//
// It reuses `network::get_with_status()` so it can read fingerprints that live in
// error pages (a dangling GitHub Pages host answers 404 with its takeover string).

use serde::Serialize;

use crate::network;

pub struct Service {
    pub name: &'static str,
    pub cnames: &'static [&'static str],
    pub fingerprints: &'static [&'static str],
    // True when the hostname no longer resolving is itself the takeover signal.
    pub nxdomain: bool,
}

pub const SERVICES: &[Service] = &[
    Service { name: "GitHub Pages", cnames: &[".github.io"],
        fingerprints: &["There isn't a GitHub Pages site here"], nxdomain: false },
    Service { name: "AWS S3", cnames: &[".s3.amazonaws.com", ".s3-website", ".amazonaws.com"],
        fingerprints: &["NoSuchBucket", "The specified bucket does not exist"], nxdomain: false },
    Service { name: "Heroku", cnames: &[".herokuapp.com", ".herokudns.com", ".herokussl.com"],
        fingerprints: &["No such app", "herokucdn.com/error-pages/no-such-app.html"], nxdomain: false },
    Service { name: "Fastly", cnames: &[".fastly.net"],
        fingerprints: &["Fastly error: unknown domain"], nxdomain: false },
    Service { name: "Shopify", cnames: &[".myshopify.com"],
        fingerprints: &["Sorry, this shop is currently unavailable"], nxdomain: false },
    Service { name: "Surge.sh", cnames: &[".surge.sh"],
        fingerprints: &["project not found"], nxdomain: false },
    Service { name: "Bitbucket", cnames: &[".bitbucket.io"],
        fingerprints: &["Repository not found"], nxdomain: false },
    Service { name: "Ghost", cnames: &[".ghost.io"],
        fingerprints: &["The thing you were looking for is no longer here", "Domain error"], nxdomain: false },
    Service { name: "Zendesk", cnames: &[".zendesk.com"],
        fingerprints: &["Help Center Closed"], nxdomain: false },
    Service { name: "Pantheon", cnames: &[".pantheonsite.io"],
        fingerprints: &["The gods are wise", "404 error unknown site!"], nxdomain: false },
    Service { name: "Tumblr", cnames: &[".domains.tumblr.com"],
        fingerprints: &["Whatever you were looking for doesn't currently exist at this address"], nxdomain: false },
    Service { name: "Microsoft Azure",
        cnames: &[".azurewebsites.net", ".cloudapp.net", ".trafficmanager.net", ".blob.core.windows.net"],
        fingerprints: &["404 Web Site not found"], nxdomain: true },
    Service { name: "Wordpress", cnames: &[".wordpress.com"],
        fingerprints: &["Do you want to register"], nxdomain: false },
];

#[derive(Serialize, Debug, Clone)]
pub struct TakeoverVerdict {
    pub target: String,
    pub vulnerable: bool,
    pub service: Option<String>,
    pub evidence: String,
}

pub fn match_service(target: &str) -> Option<&'static Service> {
    let t = target.trim().trim_end_matches('.').to_lowercase();
    SERVICES.iter().find(|s| s.cnames.iter().any(|c| t.contains(c)))
}

pub fn body_indicates_takeover(service: &Service, body: &str) -> bool {
    let low = body.to_lowercase();
    service.fingerprints.iter().any(|fp| low.contains(&fp.to_lowercase()))
}

pub fn check(host: &str) -> TakeoverVerdict {
    let t = host.trim().trim_end_matches('.').to_lowercase();
    let service = match match_service(&t) {
        Some(s) => s,
        None => {
            return TakeoverVerdict {
                target: t,
                vulnerable: false,
                service: None,
                evidence: "Target is not a known takeover-prone service.".to_string(),
            }
        }
    };

    match network::get_with_status(&format!("https://{}/", t)) {
        Ok((_status, body)) => {
            if body_indicates_takeover(service, &body) {
                TakeoverVerdict {
                    target: t,
                    vulnerable: true,
                    service: Some(service.name.to_string()),
                    evidence: format!(
                        "{} returned an 'unclaimed resource' response — this host can likely be taken over.",
                        service.name
                    ),
                }
            } else {
                TakeoverVerdict {
                    target: t,
                    vulnerable: false,
                    service: Some(service.name.to_string()),
                    evidence: format!("{} target appears live/claimed.", service.name),
                }
            }
        }
        Err(e) => {
            // Couldn't reach it. For services that free the hostname, that's the signal.
            if service.nxdomain {
                TakeoverVerdict {
                    target: t,
                    vulnerable: true,
                    service: Some(service.name.to_string()),
                    evidence: format!(
                        "{} target does not resolve — the backing resource looks unclaimed.",
                        service.name
                    ),
                }
            } else {
                TakeoverVerdict {
                    target: t,
                    vulnerable: false,
                    service: Some(service.name.to_string()),
                    evidence: format!("Could not reach target ({}).", e),
                }
            }
        }
    }
}

pub fn print_human(v: &TakeoverVerdict) {
    use colored::Colorize;
    println!("\n  {} {}", "Takeover check for".cyan().bold(), v.target.white().bold());
    println!("  {}", "───────────────────────────────────────".dimmed());
    println!("    {:<10} {}", "Service".yellow(), v.service.clone().unwrap_or_else(|| "—".to_string()));
    if v.vulnerable {
        println!("    {:<10} {} {}", "Status".yellow(), "⚠".red().bold(), "VULNERABLE".red().bold());
    } else {
        println!("    {:<10} {} {}", "Status".yellow(), "✓".green().bold(), "Not vulnerable".green());
    }
    println!("    {:<10} {}", "Detail".yellow(), v.evidence);
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_services() {
        assert_eq!(match_service("myrepo.github.io").unwrap().name, "GitHub Pages");
        assert_eq!(match_service("app.herokuapp.com").unwrap().name, "Heroku");
        assert!(match_service("cdn.mycompany.com").is_none());
    }

    #[test]
    fn detects_fingerprint_in_body() {
        let gh = match_service("x.github.io").unwrap();
        assert!(body_indicates_takeover(gh, "404: There isn't a GitHub Pages site here."));
        assert!(!body_indicates_takeover(gh, "<html>My live blog</html>"));
    }
}
