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

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::network;

// Takeover fingerprints are loaded from a shared data file
// (data/takeover_fingerprints.json) kept byte-for-byte identical to PortIntel's
// copy, so the two never drift. Embedded at build time to keep the binary
// self-contained.
#[derive(Deserialize)]
pub struct Service {
    #[serde(rename = "service")]
    pub name: String,
    pub cnames: Vec<String>,
    pub fingerprints: Vec<String>,
    // True when the hostname no longer resolving is itself the takeover signal.
    pub nxdomain: bool,
}

#[derive(Deserialize)]
struct FingerprintsFile {
    services: Vec<Service>,
}

pub static SERVICES: LazyLock<Vec<Service>> = LazyLock::new(|| {
    let raw = include_str!("../data/takeover_fingerprints.json");
    serde_json::from_str::<FingerprintsFile>(raw)
        .expect("data/takeover_fingerprints.json is valid JSON")
        .services
});

#[derive(Serialize, Debug, Clone)]
pub struct TakeoverVerdict {
    pub target: String,
    pub vulnerable: bool,
    pub service: Option<String>,
    pub evidence: String,
}

pub fn match_service(target: &str) -> Option<&'static Service> {
    let t = target.trim().trim_end_matches('.').to_lowercase();
    SERVICES.iter().find(|s| s.cnames.iter().any(|c| t.contains(c.as_str())))
}

pub fn body_indicates_takeover(service: &Service, body: &str) -> bool {
    let low = body.to_lowercase();
    service.fingerprints.iter().any(|fp| low.contains(fp.to_lowercase().as_str()))
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
