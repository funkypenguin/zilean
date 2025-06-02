use regress::*;

pub trait RegexStringExt
where
    Self: Sized,
{
    fn case_insensitive(pattern: &str) -> Result<Self, Error>;

    fn find_str<'s>(&self, subject: &'s str) -> Option<StringMatch<'s>>;
    fn find_iter_str<'r, 's>(&'r self, subject: &'s str) -> StringMatches<'s, 'r, 's>;
    fn contains_match(&self, subject: &str) -> bool;

    fn replace_all(&self, subject: &str, replacement: &str) -> String;
    fn replace_all_with_captures(
        &self,
        subject: &str,
        replacement: &str,
        use_captures: bool,
    ) -> String;
}

impl RegexStringExt for Regex {
    fn case_insensitive(pattern: &str) -> Result<Self, Error> {
        let flags = Flags {
            icase: true,
            ..Default::default()
        };
        Regex::with_flags(pattern, flags)
    }

    fn find_str<'s>(&self, subject: &'s str) -> Option<StringMatch<'s>> {
        let raw_match = self.find(subject)?;
        Some(StringMatch {
            m: raw_match,
            full_input: subject,
        })
    }

    fn find_iter_str<'r, 't>(&'r self, subject: &'t str) -> StringMatches<'t, 'r, 't> {
        let raw_matches = self.find_iter(subject);
        StringMatches {
            matches: raw_matches,
            full_input: subject,
        }
    }

    fn contains_match(&self, subject: &str) -> bool {
        self.find(subject).is_some()
    }

    fn replace_all(&self, subject: &str, replacement: &str) -> String {
        self.replace_all_with_captures(subject, replacement, false)
    }

    fn replace_all_with_captures(
        &self,
        subject: &str,
        replacement: &str,
        use_captures: bool,
    ) -> String {
        let mut result = String::with_capacity(subject.len());
        let mut last_match_end = 0;

        for m in self.find_iter(subject) {
            result.push_str(&subject[last_match_end..m.start()]);

            if use_captures {
                let mut chars = replacement.chars().peekable();
                while let Some(c) = chars.next() {
                    if c == '\\' {
                        if let Some(group_num) = chars.next().and_then(|d| d.to_digit(10)) {
                            if let Some(group) = m
                                .captures
                                .get(group_num as usize - 1)
                                .and_then(|r| r.as_ref())
                            {
                                result.push_str(&subject[group.clone()]);
                            }
                        } else {
                            result.push('\\');
                        }
                    } else {
                        result.push(c);
                    }
                }
            } else {
                result.push_str(replacement);
            }

            last_match_end = m.end();
        }

        result.push_str(&subject[last_match_end..]);
        result
    }
}

pub struct StringMatch<'a> {
    pub m: Match,
    pub full_input: &'a str,
}

impl<'a> StringMatch<'a> {
    pub fn as_str(&self) -> &str {
        &self.full_input[self.m.range()]
    }

    /// Doesn't panic if index out of bounds, just returns None
    #[inline]
    fn safe_group(&self, idx: usize) -> Option<Range> {
        if idx == 0 {
            Some(self.m.range.clone())
        } else {
            self.m.captures.get(idx - 1).cloned().flatten()
        }
    }

    pub fn group(&self, idx: usize) -> Option<GroupMatch<'a>> {
        if let Some(group_range) = self.safe_group(idx) {
            Some(GroupMatch {
                range: group_range,
                full_input: self.full_input,
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.m.start()
    }
}

pub struct GroupMatch<'a> {
    pub range: Range,
    pub full_input: &'a str,
}

impl<'a> GroupMatch<'a> {
    pub fn as_str(&self) -> &'a str {
        &self.full_input[self.range.clone()]
    }
}

pub struct StringMatches<'a, 'r, 't> {
    pub matches: Matches<'r, 't>,
    pub full_input: &'a str,
}

impl<'a, 'r, 't> Iterator for StringMatches<'a, 'r, 't> {
    type Item = StringMatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.matches.next().map(|m| StringMatch {
            m,
            full_input: self.full_input,
        })
    }
}
