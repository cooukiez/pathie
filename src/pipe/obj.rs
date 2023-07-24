pub const BASE_CUBE_VERT: [(f32, f32, f32); 24] = [
    (-0.5, 0.5, -0.5),
    (-0.5, -0.5, -0.5),
    (0.5, -0.5, -0.5),
    (0.5, 0.5, -0.5),
    (-0.5, 0.5, 0.5),
    (-0.5, -0.5, 0.5),
    (0.5, -0.5, 0.5),
    (0.5, 0.5, 0.5),
    (0.5, 0.5, -0.5),
    (0.5, -0.5, -0.5),
    (0.5, -0.5, 0.5),
    (0.5, 0.5, 0.5),
    (-0.5, 0.5, -0.5),
    (-0.5, -0.5, -0.5),
    (-0.5, -0.5, 0.5),
    (-0.5, 0.5, 0.5),
    (-0.5, 0.5, 0.5),
    (-0.5, 0.5, -0.5),
    (0.5, 0.5, -0.5),
    (0.5, 0.5, 0.5),
    (-0.5, -0.5, 0.5),
    (-0.5, -0.5, -0.5),
    (0.5, -0.5, -0.5),
    (0.5, -0.5, 0.5),
];

pub const BASE_CUBE_UV: [(i32, i32); 24] = [
    (0, 0),
    (0, 1),
    (1, 1),
    (1, 0),
    (0, 0),
    (0, 1),
    (1, 1),
    (1, 0),
    (0, 0),
    (0, 1),
    (1, 1),
    (1, 0),
    (0, 0),
    (0, 1),
    (1, 1),
    (1, 0),
    (0, 0),
    (0, 1),
    (1, 1),
    (1, 0),
    (0, 0),
    (0, 1),
    (1, 1),
    (1, 0),
];

pub const BASE_CUBE_IDX: [i32; 36] = [
    0, 1, 3, 3, 1, 2, 4, 5, 7, 7, 5, 6, 8, 9, 11, 11, 9, 10, 12, 13, 15, 15, 13, 14, 16, 17, 19,
    19, 17, 18, 20, 21, 23, 23, 21, 22,
];
