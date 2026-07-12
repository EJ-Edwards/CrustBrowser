// --- ipcheck.rs ---
// Turns CrustBrowser's HTTP layer into an "IP intel" check: given an IP address,
// it asks RDAP (the structured, modern replacement for WHOIS) who owns the
// network, then decides whether that owner is a known hosting/CDN provider.
//
// This is what PortIntel needs to cut DNS-change false alarms: if an A record
// moves to a new IP that still belongs to the same hosting provider (or a known
// CDN like Cloudflare/AWS), it's routine infrastructure churn — not a hijack.
//
// It reuses `network::get()` for the actual HTTP request and `serde_json`
// (already a dependency) to parse the RDAP response.

use serde::Serialize;
use serde_json::Value;

use crate::network;

// The structured result of an IP lookup. `Serialize` lets the non-interactive
// `ipcheck <ip> --json` mode emit this straight to stdout for another program
// (e.g. PortIntel's monitor pipeline) to consume.
#[derive(Serialize, Debug, Clone)]
pub struct IpInfo {
    pub ip: String,
    pub network_name: Option<String>, // RDAP "name", e.g. "CLOUDFLARENET", "AMAZON-02"
    pub handle: Option<String>,       // RDAP registry handle
    pub org: Option<String>,          // Registered org, e.g. "Cloudflare, Inc."
    pub cidr: Option<String>,         // The owning block, e.g. "104.16.0.0/13"
    pub country: Option<String>,      // ISO country code
    pub known_host: Option<String>,   // Canonical provider name if recognised
    pub is_known_host: bool,          // Convenience flag mirroring known_host.is_some()
    pub source: String,               // Which RDAP source answered
}

// Known hosting / CDN / cloud operators, matched as lowercase substrings against
// the RDAP network name and org. Kept deliberately broad — a match means "this
// is infrastructure, a record change here is probably routine", so we'd rather
// recognise a provider than miss one. First match wins, so order by specificity.
const KNOWN_HOSTS: &[(&str, &str)] = &[
    ("cloudflare", "Cloudflare"),
    ("cloudfront", "Amazon CloudFront"),
    ("amazon", "Amazon AWS"),
    ("aws", "Amazon AWS"),
    ("google", "Google Cloud"),
    ("gogl", "Google Cloud"),
    ("goog", "Google Cloud"),
    ("microsoft", "Microsoft Azure"),
    ("azure", "Microsoft Azure"),
    ("msft", "Microsoft Azure"),
    ("fastly", "Fastly"),
    ("akamai", "Akamai"),
    ("linode", "Akamai (Linode)"),
    ("vercel", "Vercel"),
    ("netlify", "Netlify"),
    ("digitalocean", "DigitalOcean"),
    ("digital ocean", "DigitalOcean"),
    ("ovh", "OVH"),
    ("hetzner", "Hetzner"),
    ("github", "GitHub"),
    ("oracle", "Oracle Cloud"),
    ("render", "Render"),
    ("heroku", "Heroku"),
    ("digital realty", "Digital Realty"),
];

// Look up an IP over the network. Validates the address, queries RDAP, and
// parses the response. Network errors and unallocated IPs surface as Err.
pub fn lookup(ip: &str) -> Result<IpInfo, String> {
    let ip = ip.trim();
    // Reject anything that isn't a valid IPv4/IPv6 literal before hitting the
    // network — avoids leaking arbitrary strings into the RDAP URL.
    if ip.parse::<std::net::IpAddr>().is_err() {
        return Err(format!("'{}' is not a valid IP address", ip));
    }

    let url = format!("https://rdap.org/ip/{}", ip);
    let body = network::get(&url)?;
    parse_rdap(ip, &body)
}

// Parse an RDAP IP-network JSON body into IpInfo. Pure (no network) so it can be
// unit-tested against fixtures. RDAP responses vary between the regional
// registries, so every field is extracted defensively.
pub fn parse_rdap(ip: &str, body: &str) -> Result<IpInfo, String> {
    let v: Value =
        serde_json::from_str(body).map_err(|e| format!("RDAP response was not valid JSON: {}", e))?;

    let network_name = v.get("name").and_then(Value::as_str).map(str::to_string);
    let handle = v.get("handle").and_then(Value::as_str).map(str::to_string);
    let country = v.get("country").and_then(Value::as_str).map(str::to_string);
    let org = extract_org(&v);
    let cidr = extract_cidr(&v);

    let known_host = classify(network_name.as_deref(), org.as_deref(), handle.as_deref());

    Ok(IpInfo {
        ip: ip.to_string(),
        network_name,
        handle,
        org,
        cidr,
        country,
        is_known_host: known_host.is_some(),
        known_host,
        source: "rdap.org".to_string(),
    })
}

// Match the network name / org / handle against the known-provider table.
fn classify(name: Option<&str>, org: Option<&str>, handle: Option<&str>) -> Option<String> {
    let haystack = format!(
        "{} {} {}",
        name.unwrap_or(""),
        org.unwrap_or(""),
        handle.unwrap_or("")
    )
    .to_lowercase();
    for (needle, canonical) in KNOWN_HOSTS {
        if haystack.contains(needle) {
            return Some((*canonical).to_string());
        }
    }
    None
}

// Pull the registered org name out of the RDAP `entities` → vCard array. The
// vCard is JSON-encoded jCard: ["vcard", [ ["fn", {}, "text", "Cloudflare, Inc."], ... ]].
// We search entities (and their nested sub-entities) for the first "fn" value.
fn extract_org(v: &Value) -> Option<String> {
    let entities = v.get("entities").and_then(Value::as_array)?;
    org_from_entities(entities)
}

fn org_from_entities(entities: &[Value]) -> Option<String> {
    for entity in entities {
        if let Some(name) = fn_from_vcard(entity.get("vcardArray")) {
            return Some(name);
        }
        // Some registries nest the org under a sub-entity.
        if let Some(sub) = entity.get("entities").and_then(Value::as_array) {
            if let Some(name) = org_from_entities(sub) {
                return Some(name);
            }
        }
    }
    None
}

fn fn_from_vcard(vcard: Option<&Value>) -> Option<String> {
    // vcardArray = ["vcard", [ [prop, params, type, value], ... ]]
    let props = vcard.and_then(Value::as_array)?.get(1)?.as_array()?;
    for prop in props {
        let arr = prop.as_array()?;
        if arr.first().and_then(Value::as_str) == Some("fn") {
            if let Some(val) = arr.get(3).and_then(Value::as_str) {
                let val = val.trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}

// Build a human-readable CIDR from either cidr0_cidrs or start/end addresses.
fn extract_cidr(v: &Value) -> Option<String> {
    if let Some(cidrs) = v.get("cidr0_cidrs").and_then(Value::as_array) {
        if let Some(first) = cidrs.first() {
            let prefix = first
                .get("v4prefix")
                .or_else(|| first.get("v6prefix"))
                .and_then(Value::as_str);
            let length = first.get("length").and_then(Value::as_u64);
            if let (Some(p), Some(l)) = (prefix, length) {
                return Some(format!("{}/{}", p, l));
            }
        }
    }
    let start = v.get("startAddress").and_then(Value::as_str);
    let end = v.get("endAddress").and_then(Value::as_str);
    match (start, end) {
        (Some(s), Some(e)) => Some(format!("{} – {}", s, e)),
        (Some(s), None) => Some(s.to_string()),
        _ => None,
    }
}

// Pretty terminal output for the interactive `ip <addr>` command and the
// non-JSON CLI mode.
pub fn print_human(info: &IpInfo) {
    use colored::Colorize;
    println!("\n  {} {}", "IP intel for".cyan().bold(), info.ip.white().bold());
    println!("  {}", "───────────────────────────────────────".dimmed());
    let row = |label: &str, val: &Option<String>| {
        println!(
            "    {:<14} {}",
            label.yellow(),
            val.clone().unwrap_or_else(|| "—".to_string())
        );
    };
    row("Network", &info.network_name);
    row("Organisation", &info.org);
    row("CIDR block", &info.cidr);
    row("Country", &info.country);
    row("Handle", &info.handle);
    match &info.known_host {
        Some(provider) => println!(
            "    {:<14} {} {}",
            "Hosting".yellow(),
            "✓".green().bold(),
            format!("Known provider: {}", provider).green()
        ),
        None => println!(
            "    {:<14} {} {}",
            "Hosting".yellow(),
            "?".red().bold(),
            "Not a recognised hosting/CDN provider".red()
        ),
    }
    println!("  {}\n", format!("source: {}", info.source).dimmed());
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trimmed ARIN-style RDAP response for one of Google's networks.
    const GOOGLE_RDAP: &str = r#"{
        "objectClassName": "ip network",
        "handle": "NET-8-8-8-0-1",
        "startAddress": "8.8.8.0",
        "endAddress": "8.8.8.255",
        "name": "GOGL",
        "country": "US",
        "cidr0_cidrs": [{ "v4prefix": "8.8.8.0", "length": 24 }],
        "entities": [{
            "objectClassName": "entity",
            "handle": "GOGL",
            "roles": ["registrant"],
            "vcardArray": ["vcard", [
                ["version", {}, "text", "4.0"],
                ["fn", {}, "text", "Google LLC"]
            ]]
        }]
    }"#;

    // A residential ISP block — the kind of destination a real hijack points to.
    const RESIDENTIAL_RDAP: &str = r#"{
        "objectClassName": "ip network",
        "handle": "NET-203-0-113-0-1",
        "startAddress": "203.0.113.0",
        "endAddress": "203.0.113.255",
        "name": "EXAMPLE-DSL-POOL",
        "country": "RU",
        "cidr0_cidrs": [{ "v4prefix": "203.0.113.0", "length": 24 }],
        "entities": [{
            "vcardArray": ["vcard", [
                ["fn", {}, "text", "Example Regional Telecom"]
            ]]
        }]
    }"#;

    #[test]
    fn parses_and_classifies_known_host() {
        let info = parse_rdap("8.8.8.8", GOOGLE_RDAP).unwrap();
        assert_eq!(info.network_name.as_deref(), Some("GOGL"));
        assert_eq!(info.org.as_deref(), Some("Google LLC"));
        assert_eq!(info.cidr.as_deref(), Some("8.8.8.0/24"));
        assert_eq!(info.country.as_deref(), Some("US"));
        assert_eq!(info.known_host.as_deref(), Some("Google Cloud"));
        assert!(info.is_known_host);
    }

    #[test]
    fn unknown_host_is_not_flagged() {
        let info = parse_rdap("203.0.113.10", RESIDENTIAL_RDAP).unwrap();
        assert_eq!(info.org.as_deref(), Some("Example Regional Telecom"));
        assert_eq!(info.known_host, None);
        assert!(!info.is_known_host);
    }

    #[test]
    fn rejects_invalid_ip() {
        assert!(lookup("not-an-ip").is_err());
        assert!(lookup("999.999.1.1").is_err());
    }

    #[test]
    fn errors_on_garbage_json() {
        assert!(parse_rdap("8.8.8.8", "<html>nope</html>").is_err());
    }
}
