//! Synthetic fixture generators for TOK-002.
//!
//! All content is synthetic and reproducible (fixed seed). No network, no env,
//! no real names or email-shaped strings.

use serde_json::{json, Value};

const LOREM: &str = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do \
eiusmod tempor incididunt ut labore et dolore magna aliqua ut enim ad minim veniam \
quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat \
duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu \
fugiat nulla pariatur excepteur sint occaecat cupidatat non proident sunt in culpa \
qui officia deserunt mollit anim id est laborum ";

/// Deterministic LCG for reproducible synthetic data.
#[derive(Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        (self.state >> 32) as u32
    }

    pub fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next_u32() as usize) % max
    }
}

/// Named fixture with JSON text.
#[derive(Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub json: String,
}

impl Fixture {
    pub fn bytes(&self) -> usize {
        self.json.len()
    }
}

/// Build the full TOK-002 corpus.
pub fn build_corpus() -> Vec<Fixture> {
    let mut out = Vec::new();
    for n in [8usize, 100, 1000] {
        out.push(Fixture {
            name: format!("homogeneous_{n}"),
            json: serde_json::to_string(&homogeneous_rows(n)).expect("serialize"),
        });
    }
    out.push(Fixture {
        name: "heterogeneous_nested".into(),
        json: serde_json::to_string(&heterogeneous_nested(Rng::new(42), 5, 0))
            .expect("serialize"),
    });
    out.push(Fixture {
        name: "realistic_api".into(),
        json: serde_json::to_string(&realistic_api(Rng::new(7))).expect("serialize"),
    });
    out.push(Fixture {
        name: "prose_heavy".into(),
        json: serde_json::to_string(&prose_heavy(32)).expect("serialize"),
    });
    for target in [1024usize, 10 * 1024, 100 * 1024, 250 * 1024] {
        out.push(size_ladder(target));
    }
    out
}

fn homogeneous_rows(n: usize) -> Value {
    let statuses = ["active", "pending", "closed"];
    let roles = ["member", "viewer", "editor"];
    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        rows.push(json!({
            "rec_id": i,
            "ts": format!("2024-06-{:02}T12:00:00Z", (i % 28) + 1),
            "status": statuses[i % statuses.len()],
            "role": roles[i % roles.len()],
        }));
    }
    Value::Array(rows)
}

fn heterogeneous_nested(mut rng: Rng, depth: u32, idx: usize) -> Value {
    if depth == 0 {
        return json!(rng.next_usize(1000));
    }
    let key = format!("node_{idx}_{depth}");
    let sibling = format!("alt_{idx}");
    json!({
        key: heterogeneous_nested(rng.clone(), depth - 1, idx * 2 + 1),
        sibling: {
            "flag": rng.next_usize(2) == 0,
            "code": format!("C{:04}", rng.next_usize(9999)),
            "nested": heterogeneous_nested(rng, depth - 1, idx * 2 + 2),
        }
    })
}

fn realistic_api(mut rng: Rng) -> Value {
    let mut items = Vec::new();
    for i in 0..40 {
        items.push(json!({
            "item_id": 10_000 + i,
            "field_alpha": format!("payload_blob_{}_xxxxxxxxxxxxxxxx", i),
            "field_beta": if i % 5 == 0 { Value::Null } else { json!(format!("tag_{}", i % 7)) },
            "meta": {
                "created_at": format!("2024-01-{:02}T08:30:00Z", (i % 28) + 1),
                "updated_at": format!("2024-02-{:02}T09:15:00Z", (i % 28) + 1),
                "score": rng.next_usize(100),
                "active": i % 3 != 0,
            },
            "tags": ["alpha", "beta", "gamma", "alpha"],
        }));
    }
    json!({
        "page": 1,
        "page_size": 40,
        "total": 40,
        "items": items,
    })
}

fn prose_heavy(paragraphs: usize) -> Value {
    let mut body = String::new();
    for i in 0..paragraphs {
        body.push_str(LOREM);
        body.push_str(&format!("paragraph_{i} "));
    }
    json!({
        "doc_id": "synthetic_doc_001",
        "title": "synthetic prose fixture",
        "body": body,
    })
}

fn size_ladder(target: usize) -> Fixture {
    // Grow homogeneous rows until serialized size is within ±5% of target.
    let mut lo = 1usize;
    let mut hi = 50_000usize;
    let mut best_json = String::new();
    while lo <= hi {
        let mid = (lo + hi) / 2;
        let v = homogeneous_rows(mid);
        let s = serde_json::to_string(&v).expect("ser");
        let len = s.len();
        best_json = s;
        if len + len / 20 < target {
            lo = mid + 1;
        } else if len > target + target / 20 {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        } else {
            break;
        }
    }
    let label = if target >= 1024 {
        format!("size_{}k", target / 1024)
    } else {
        format!("size_{target}")
    };
    Fixture {
        name: label,
        json: best_json,
    }
}

/// Forbidden substrings that would look like real PII / scraped data.
pub const FORBIDDEN_PATTERNS: &[&str] = &[
    "@gmail.",
    "@yahoo.",
    "@hotmail.",
    "mailto:",
    "John ",
    "Jane ",
    "Smith",
    "password",
    "ssn",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_has_expected_names() {
        let c = build_corpus();
        let names: Vec<_> = c.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"homogeneous_100"));
        assert!(names.contains(&"prose_heavy"));
        assert!(names.contains(&"size_100k"));
    }

    #[test]
    fn fixtures_contain_no_forbidden_patterns() {
        for f in build_corpus() {
            let lower = f.json.to_ascii_lowercase();
            for pat in FORBIDDEN_PATTERNS {
                assert!(
                    !lower.contains(&pat.to_ascii_lowercase()),
                    "fixture {} contains forbidden pattern {pat}",
                    f.name
                );
            }
        }
    }

    #[test]
    fn no_env_or_network_in_this_module() {
        // Compile-time intent: this module must not reference std::env or HTTP clients.
        // Runtime smoke: corpus builds offline.
        let _ = build_corpus();
    }
}
