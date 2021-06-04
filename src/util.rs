#[derive(Debug, PartialEq, Copy, Clone)]
//https://jonasjacek.github.io/colors/
pub enum Color {
    Black = 0,
    Cyan = 44,
    Purple = 90,
    Green = 22,
    Red = 1,
    Blue = 21,
    Orange = 202,
    Yellow = 190,
    DarkCyan = 36,
    DarkPurple = 5,
    DarkGreen = 2,
    DarkRed = 9,
    DarkBlue = 4,
    DarkOrange = 94,
    DarkYellow = 11,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Direction {
    Left,
    Right,
}
