use std::collections::HashSet;

pub struct SolidSeg {
    width: i16,
    range_set: HashSet<i16>,
}

impl SolidSeg {
    pub fn new(width: i16) -> Self {
        Self {
            width,
            range_set: (0..width).collect(),
        }
    }

    pub fn initialize(&mut self) {
        self.range_set = (0..self.width).collect();
    }

    pub fn get_renderable_ranges(&self, range: (i16, i16)) -> Vec<(i16, i16)> {
        let mut renderable_ranges: Vec<(i16, i16)> = Vec::new();
        let range_set: HashSet<i16> = (range.0..range.1 + 1).collect();
        let intersection = {
            let mut intersection = range_set
                .intersection(&self.range_set)
                .cloned()
                .collect::<Vec<i16>>();
            intersection.sort();
            intersection
        };
        if range_set.len() == intersection.len() {
            // 全ての範囲を描画できる
            renderable_ranges.push(range);
        } else if !intersection.is_empty() {
            // 描画範囲を分割する
            let mut start = intersection[0];
            let mut end = start;
            for &value in &intersection[1..] {
                if value == end + 1 {
                    end = value;
                } else {
                    renderable_ranges.push((start, end));
                    start = value;
                    end = value;
                }
            }
            renderable_ranges.push((start, end));
        }
        renderable_ranges
    }

    pub fn set_renderable_range(&mut self, range: (i16, i16)) {
        let range_set: HashSet<i16> = (range.0..range.1 + 1).collect();
        self.range_set = self.range_set.difference(&range_set).cloned().collect();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::solidseg::SolidSeg;

    #[test]
    fn test_solid_seg() {
        let mut solid_seg = SolidSeg::new(100);
        for (range, expected) in [
            ((0, 20), vec![(0, 20)]),
            ((10, 30), vec![(21, 30)]),
            ((80, 99), vec![(80, 99)]),
            ((75, 85), vec![(75, 79)]),
            ((50, 55), vec![(50, 55)]),
            ((60, 65), vec![(60, 65)]),
            ((40, 70), vec![(40, 49), (56, 59), (66, 70)]),
        ] {
            assert_eq!(solid_seg.get_renderable_ranges(range), expected);
            solid_seg.set_renderable_range(range);
        }
    }
}
