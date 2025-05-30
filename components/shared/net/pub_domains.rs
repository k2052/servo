/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implementation of public domain matching.
//!
//! The list is a file located on the `resources` folder and loaded once on first need.
//!
//! The list can be updated with `./mach update-pub-domains` from this source:
//! <https://publicsuffix.org/list/>
//!
//! This implementation is not strictly following the specification of the list. Wildcards are not
//! restricted to appear only in the leftmost position, but the current list has no such cases so
//! we don't need to make the code more complex for it. The `mach` update command makes sure that
//! those cases are not present.

use std::collections::HashSet;
use std::iter::FromIterator;
use std::sync::LazyLock;

use embedder_traits::resources::{self, Resource};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use malloc_size_of_derive::MallocSizeOf;
use servo_url::{Host, ImmutableOrigin, ServoUrl};

#[derive(Clone, Debug, Default, MallocSizeOf)]
pub struct PubDomainRules {
    rules: HashSet<String>,
    wildcards: HashSet<String>,
    exceptions: HashSet<String>,
}

static PUB_DOMAINS: LazyLock<PubDomainRules> = LazyLock::new(load_pub_domains);

pub fn public_suffix_list_size_of(ops: &mut MallocSizeOfOps) -> usize {
    PUB_DOMAINS.size_of(ops)
}

impl<'a> FromIterator<&'a str> for PubDomainRules {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        let mut result = PubDomainRules::default();
        for item in iter {
            if let Some(stripped) = item.strip_prefix('!') {
                result.exceptions.insert(String::from(stripped));
            } else if let Some(stripped) = item.strip_prefix("*.") {
                result.wildcards.insert(String::from(stripped));
            } else {
                result.rules.insert(String::from(item));
            }
        }
        result
    }
}

impl PubDomainRules {
    pub fn parse(content: &str) -> PubDomainRules {
        content
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|s| !s.starts_with("//"))
            .collect()
    }
    fn suffix_pair<'a>(&self, domain: &'a str) -> (&'a str, &'a str) {
        let domain = domain.trim_start_matches('.');
        let mut suffix = domain;
        let mut prev_suffix = domain;
        for (index, _) in domain.match_indices('.') {
            let next_suffix = &domain[index + 1..];
            if self.exceptions.contains(suffix) {
                return (next_suffix, suffix);
            }
            if self.wildcards.contains(next_suffix) || self.rules.contains(suffix) {
                return (suffix, prev_suffix);
            }
            prev_suffix = suffix;
            suffix = next_suffix;
        }
        (suffix, prev_suffix)
    }
    pub fn public_suffix<'a>(&self, domain: &'a str) -> &'a str {
        let (public, _) = self.suffix_pair(domain);
        public
    }
    pub fn registrable_suffix<'a>(&self, domain: &'a str) -> &'a str {
        let (_, registrable) = self.suffix_pair(domain);
        registrable
    }
    pub fn is_public_suffix(&self, domain: &str) -> bool {
        // Speeded-up version of
        // domain != "" &&
        // self.public_suffix(domain) == domain.
        let domain = domain.trim_start_matches('.');
        match domain.find('.') {
            None => !domain.is_empty(),
            Some(index) => {
                !self.exceptions.contains(domain) && self.wildcards.contains(&domain[index + 1..]) ||
                    self.rules.contains(domain)
            },
        }
    }
    pub fn is_registrable_suffix(&self, domain: &str) -> bool {
        // Speeded-up version of
        // self.public_suffix(domain) != domain &&
        // self.registrable_suffix(domain) == domain.
        let domain = domain.trim_start_matches('.');
        match domain.find('.') {
            None => false,
            Some(index) => {
                self.exceptions.contains(domain) ||
                    !self.wildcards.contains(&domain[index + 1..]) &&
                        !self.rules.contains(domain) &&
                        self.is_public_suffix(&domain[index + 1..])
            },
        }
    }
}

fn load_pub_domains() -> PubDomainRules {
    PubDomainRules::parse(&resources::read_string(Resource::DomainList))
}

pub fn pub_suffix(domain: &str) -> &str {
    PUB_DOMAINS.public_suffix(domain)
}

pub fn reg_suffix(domain: &str) -> &str {
    PUB_DOMAINS.registrable_suffix(domain)
}

pub fn is_pub_domain(domain: &str) -> bool {
    PUB_DOMAINS.is_public_suffix(domain)
}

pub fn is_reg_domain(domain: &str) -> bool {
    PUB_DOMAINS.is_registrable_suffix(domain)
}

/// The registered domain name (aka eTLD+1) for a URL.
/// Returns None if the URL has no host name.
/// Returns the registered suffix for the host name if it is a domain.
/// Leaves the host name alone if it is an IP address.
pub fn reg_host(url: &ServoUrl) -> Option<Host> {
    match url.origin() {
        ImmutableOrigin::Tuple(_, Host::Domain(domain), _) => {
            Some(Host::Domain(String::from(reg_suffix(&domain))))
        },
        ImmutableOrigin::Tuple(_, ip, _) => Some(ip),
        ImmutableOrigin::Opaque(_) => None,
    }
}
