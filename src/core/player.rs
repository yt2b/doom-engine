use crate::core::map::Rect;
use crate::core::math::{Line, Vector2};

pub const PLAYER_FOV: f32 = 90.0;
const VIEW_HEIGHT: f32 = 41.0;

pub struct Player {
    pub pos: Vector2,
    pub angle: f32,
    pub fov: f32,
    pub h_fov: f32,
    pub view_height: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            pos: Vector2::new(x, y),
            angle,
            fov: PLAYER_FOV,
            h_fov: PLAYER_FOV / 2.0,
            view_height: VIEW_HEIGHT,
        }
    }

    pub fn move_angle(&mut self, angle: f32) {
        self.angle = norm_angle(self.angle + angle);
    }

    pub fn move_pos(&mut self, distance: f32) {
        self.pos = self.pos + Vector2::new(distance, 0.0).rotate(self.angle);
    }

    pub fn to_insight_angle(&self, pos: Vector2) -> f32 {
        // プレイヤー座標からの相対角度に変換する
        let angle = (pos - self.pos).angle();
        // プレイヤーの視界からの相対角度に変換する。正面を0度として360度の範囲
        norm_angle(angle - self.angle)
    }

    pub fn to_fov_line_angle(&self, line: Line) -> Option<(f32, f32)> {
        // プレイヤー座標からの相対角度
        let s_angle = (line.start - self.pos).angle();
        let e_angle = (line.end - self.pos).angle();
        let span = norm_angle(s_angle - e_angle);
        // プレイヤーの方を向いている線分は常にs_angle > engleとなる
        // プレイヤーの方を向いていない線分か、プレイヤーの背後にある線分は見えない
        if norm_angle(span) >= 180.0 {
            return None;
        }
        // プレイヤーの視界からの相対角度
        let s_insight_angle = self.to_insight_angle(line.start);
        let e_insight_angle = self.to_insight_angle(line.end);
        match (
            is_insight_fov(s_insight_angle, self.h_fov),
            is_insight_fov(e_insight_angle, self.h_fov),
        ) {
            // 始点と終点が視野内にある
            (true, true) => Some((to_fov_angle(s_insight_angle), to_fov_angle(e_insight_angle))),
            // 始点が視野内、終点が視野外にある
            (true, false) => Some((to_fov_angle(s_insight_angle), -self.h_fov)),
            // 始点が視野外、終点が視野内にある
            (false, true) => Some((self.h_fov, to_fov_angle(e_insight_angle))),
            // 両端点が視野外にある
            (false, false) => {
                // 線分の途中が視野内にあるか
                (s_insight_angle - self.h_fov < span).then(|| (self.h_fov, -self.h_fov))
            }
        }
    }

    pub fn is_insight_line(&self, line: Line) -> bool {
        // プレイヤー座標からの相対角度
        let s_angle = (line.start - self.pos).angle();
        let e_angle = (line.end - self.pos).angle();
        let span = norm_angle(s_angle - e_angle);
        // プレイヤーの方を向いている線分は常にs_angle > engleとなる
        // プレイヤーの方を向いていない線分か、プレイヤーの背後にある線分は見えない
        if span >= 180.0 {
            return false;
        }
        // プレイヤーの視界からの相対角度
        let s_insight_angle = self.to_insight_angle(line.start);
        let e_insight_angle = self.to_insight_angle(line.end);
        match (
            is_insight_fov(s_insight_angle, self.h_fov),
            is_insight_fov(e_insight_angle, self.h_fov),
        ) {
            // 線分の途中が視野内にあるか
            (false, false) => s_insight_angle - self.h_fov < span,
            // 少なくともどちらかの端点が視野内にある
            _ => true,
        }
    }

    pub fn is_insight_rect(&self, rect: &Rect) -> bool {
        let a = Vector2::new(rect.left as f32, rect.top as f32);
        let b = Vector2::new(rect.right as f32, rect.top as f32);
        let c = Vector2::new(rect.right as f32, rect.bottom as f32);
        let d = Vector2::new(rect.left as f32, rect.bottom as f32);
        let ad = Line::new(a, d);
        let dc = Line::new(d, c);
        let cb = Line::new(c, b);
        let ba = Line::new(b, a);
        let position_x = get_position(self.pos.x, rect.left as f32, rect.right as f32);
        let position_y = get_position(self.pos.y, rect.bottom as f32, rect.top as f32);
        let lines = match (position_x, position_y) {
            (Position::Min, Position::Min) => vec![ad, dc], // 左下
            (Position::Min, Position::Inside) => vec![ad],  // 左
            (Position::Min, Position::Max) => vec![ba, ad], // 左上
            (Position::Inside, Position::Min) => vec![dc],  // 下
            (Position::Inside, Position::Inside) => vec![], // 内部
            (Position::Inside, Position::Max) => vec![ba],  // 上
            (Position::Max, Position::Min) => vec![dc, cb], // 右下
            (Position::Max, Position::Inside) => vec![cb],  // 右
            (Position::Max, Position::Max) => vec![cb, ba], // 右上
        };
        if lines.is_empty() {
            true
        } else {
            lines.into_iter().any(|line| self.is_insight_line(line))
        }
    }

    pub fn set_height(&mut self, height: f32) {
        self.view_height = height + VIEW_HEIGHT;
    }
}

fn get_position(x: f32, min: f32, max: f32) -> Position {
    if x < min {
        Position::Min
    } else if x >= max {
        Position::Max
    } else {
        Position::Inside
    }
}

// angleがFOVの範囲にあるか
pub fn is_insight_fov(angle: f32, h_fov: f32) -> bool {
    (0.0 <= angle && angle <= h_fov) || (360.0 - h_fov <= angle && angle <= 360.0)
}

// 正面を0度として -180~180度の範囲に変換する
fn to_fov_angle(angle: f32) -> f32 {
    if angle <= 180.0 { angle } else { angle - 360.0 }
}

fn norm_angle(angle: f32) -> f32 {
    let normed_angle = angle % 360.0;
    if normed_angle < 0.0 {
        normed_angle + 360.0
    } else {
        normed_angle
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Position {
    Min,
    Inside,
    Max,
}

#[cfg(test)]
mod tests {
    use crate::core::{
        math::{Line, Vector2},
        player::{Player, Position, get_position, is_insight_fov},
    };

    fn create_line(x1: f32, y1: f32, x2: f32, y2: f32) -> Line {
        Line::new(Vector2::new(x1, y1), Vector2::new(x2, y2))
    }

    fn reverse_line(line: Line) -> Line {
        Line::new(line.end, line.start)
    }

    #[test]
    fn test_to_insight_angle() {
        let player = Player::new(0.0, 0.0, 90.0);
        let test_cases = [
            (Vector2::new(0.0, 1.0), 0.0),
            (Vector2::new(-1.0, 1.0), 45.0),
            (Vector2::new(-1.0, 0.0), 90.0),
            (Vector2::new(-1.0, -1.0), 135.0),
            (Vector2::new(0.0, -1.0), 180.0),
            (Vector2::new(1.0, -1.0), 225.0),
            (Vector2::new(1.0, 0.0), 270.0),
            (Vector2::new(1.0, 1.0), 315.0),
        ];

        for (pos, expected_angle) in test_cases {
            assert_eq!(player.to_insight_angle(pos), expected_angle);
        }
    }

    #[test]
    fn test_to_fov_line_angle() {
        let player = Player::new(0.0, 0.0, 90.0);
        // 視野内にある線分
        for (line, expected) in [
            (
                create_line(-1.0, 2.0, 1.0, 2.0),
                Some((26.565048, -26.565063)),
            ),
            (create_line(-1.0, 2.0, 5.0, 0.0), Some((26.565048, -45.0))),
            (create_line(-5.0, 1.0, 1.0, 2.0), Some((45.0, -26.565063))),
            (create_line(1.0, 2.0, 10.0, 1.0), Some((-26.565063, -45.0))),
            (create_line(-5.0, 1.0, 5.0, 1.0), Some((45.0, -45.0))),
        ] {
            assert_eq!(player.to_fov_line_angle(line), expected);
            // 向きを反転した線分は見えない
            assert_eq!(player.to_fov_line_angle(reverse_line(line)), None);
        }
        // 視野外にある線分
        for line in [
            create_line(-4.0, -2.0, -4.0, 2.0),
            create_line(4.0, 2.0, 4.0, -2.0),
            create_line(1.0, -1.0, -1.0, 1.0),
        ] {
            assert_eq!(player.to_fov_line_angle(line), None);
            // 向きを反転した線分も見えない
            assert_eq!(player.to_fov_line_angle(reverse_line(line)), None);
        }
    }

    #[test]
    fn test_is_insight_line() {
        let player = Player::new(0.0, 0.0, 45.0);
        // 視野内にある線分
        for line in [
            create_line(-1.0, 2.0, 1.0, 2.0),
            create_line(2.0, 3.0, 1.0, 1.0),
            create_line(2.0, 1.0, 4.0, -1.0),
            create_line(-1.0, 2.0, 2.0, -1.0),
            create_line(5.0, 4.0, 5.0, -1.0),
            create_line(-5.0, 5.0, 5.0, 5.0),
        ] {
            assert_eq!(player.is_insight_line(line), true);
            // 向きを反転した線分は見えない
            assert_eq!(player.is_insight_line(reverse_line(line)), false);
        }
        // 視野外にある線分
        for line in [
            create_line(-2.0, 1.0, -2.0, 4.0),
            create_line(3.0, -1.0, 5.0, -2.0),
            create_line(-1.0, 1.0, 0.0, -1.0),
        ] {
            assert_eq!(player.is_insight_line(line), false);
            // 向きを反転した線分も見えない
            assert_eq!(player.is_insight_line(reverse_line(line)), false);
        }
    }

    #[test]
    fn test_is_insight_fov() {
        for (angle, expected) in [
            (0.0, true),
            (20.0, true),
            (45.0, true),
            (90.0, false),
            (180.0, false),
            (270.0, false),
            (315.0, true),
            (340.0, true),
        ] {
            assert_eq!(is_insight_fov(angle, 45.0), expected);
        }
    }

    #[test]
    fn test_get_position() {
        assert_eq!(get_position(0.0, 1.0, 3.0), Position::Min);
        assert_eq!(get_position(2.0, 1.0, 3.0), Position::Inside);
        assert_eq!(get_position(4.0, 1.0, 3.0), Position::Max);
    }
}
