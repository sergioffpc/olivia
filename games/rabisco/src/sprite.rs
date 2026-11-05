use olivia::{
    graphics::{Draw, Texture},
    math::Vec2,
};

/// The sprite's draw, scaled to `size` and centred.
pub(crate) fn sized(texture: &Texture, size: Vec2) -> Draw {
    Draw::new()
        .origin(Vec2::splat(0.5))
        .scale(size / texture.size())
}
