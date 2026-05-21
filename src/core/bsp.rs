use crate::core::map::{Map, Node};
use crate::core::player::Player;

// プレイヤーがいるサブセクターの高さを取得する
pub fn get_subsector_height(map: &Map, player: &Player) -> f32 {
    let mut idx = (map.nodes.len() - 1) as i16;
    while idx >= 0 {
        let node = &map.nodes[idx as usize];
        let is_on_front = is_on_front(player, node);
        if is_on_front {
            idx = node.front_child;
        } else {
            idx = node.back_child;
        }
    }
    let sub_sector = &map.subsectors[(idx & 0x7FFF) as usize];
    let seg = &map.segs[sub_sector.seg_idx as usize];
    map.sectors[seg.front_sector as usize].floor_height as f32
}

// サブセクターのインデックスをプレイヤーから見える順番で取得する
pub fn get_subsector_indices(map: &Map, player: &Player) -> Vec<usize> {
    let mut indices = Vec::new();
    traverse_node(map, player, (map.nodes.len() - 1) as i16, &mut indices);
    indices
}

fn traverse_node(map: &Map, player: &Player, idx: i16, indices: &mut Vec<usize>) {
    // 16ビット目が1ならばサブセクターのインデックス
    if idx < 0 {
        indices.push((idx & 0x7FFF) as usize);
        return;
    }
    let node = &map.nodes[idx as usize];
    let is_on_front = is_on_front(player, node);
    if is_on_front {
        traverse_node(map, player, node.front_child, indices);
        if player.is_insight_rect(&node.back_bounding) {
            traverse_node(map, player, node.back_child, indices);
        }
    } else {
        traverse_node(map, player, node.back_child, indices);
        if player.is_insight_rect(&node.front_bounding) {
            traverse_node(map, player, node.front_child, indices);
        }
    }
}

fn is_on_front(player: &Player, node: &Node) -> bool {
    let dx = player.pos.x - node.start_x as f32;
    let dy = player.pos.y - node.start_y as f32;
    dx * node.diff_y as f32 - dy * node.diff_x as f32 >= 0.0
}
