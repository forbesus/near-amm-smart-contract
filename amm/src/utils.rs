use std::cmp::max;

pub fn add_decimals(value: u128, decimals: u8) -> u128 {
    return value * 10_u128.pow(decimals as u32)
}

pub fn remove_decimals(value: u128, decimals: u8) -> u128 {
    return value / 10_u128.pow(decimals as u32)
}

pub fn calc_dy(x: u128, y: u128, dx: u128) -> u128 {
    y - (x * y / (x + dx))
}
