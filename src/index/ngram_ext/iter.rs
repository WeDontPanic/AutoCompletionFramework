pub struct NgramIter<'a> {
    s: &'a str,
    w_size: usize,
    pos: usize,
    next_start: usize,
    char_indices: Vec<usize>,
}

impl<'a> NgramIter<'a> {
    pub fn new(s: &'a str, w_size: usize) -> Self {
        let s_len = s.chars().count();
        if s_len < w_size && w_size > 0 {
            panic!("Invalid Ngram input");
        }
        let char_indices: Vec<_> = s.char_indices().map(|i| i.0).collect();
        Self {
            s,
            w_size,
            pos: 0,
            next_start: 0,
            char_indices,
        }
    }
}

impl<'a> Iterator for NgramIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let end_pos = self.pos + self.w_size;
        if end_pos > self.char_indices.len() {
            return None;
        }

        if end_pos == self.char_indices.len() {
            self.pos += 1;
            return Some(&self.s[self.next_start..]);
        }

        let end = self.char_indices[end_pos];

        let sub = &self.s[self.next_start..end];

        self.next_start = self.char_indices[self.pos + 1];
        self.pos += 1;
        Some(sub)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_window_iter_simple3() {
        let inp = "homesick";
        let w_iter = NgramIter::new(inp, 3);
        let res: Vec<_> = w_iter.collect();
        assert_eq!(res, vec!["hom", "ome", "mes", "esi", "sic", "ick"]);
    }

    #[test]
    fn test_window_iter_simple2() {
        let inp = "homesick";
        let w_iter = NgramIter::new(inp, 2);
        let res: Vec<_> = w_iter.collect();
        assert_eq!(res, vec!["ho", "om", "me", "es", "si", "ic", "ck"]);
    }

    #[test]
    fn test_window_iter_simple1() {
        let inp = "homesick";
        let w_iter = NgramIter::new(inp, 1);
        let res: Vec<_> = w_iter.collect();
        assert_eq!(res, vec!["h", "o", "m", "e", "s", "i", "c", "k"]);
    }
}
