pub fn exponential_easing(t: f32) -> f32 {
    if t == 1. {
        1.
    } else {
        1. - 2f32.powf(-10. * t)
    }
}

pub fn identity_easing(t: f32) -> f32 {
    t
}