#[derive(Debug, PartialEq, Copy, Clone)]
//https://jonasjacek.github.io/colors/
pub enum Color {
    Black = 0,
    Cyan = 44,
    Purple = 5,
    Green = 2,
    Red = 9,
    Blue = 21,
    Orange = 202,
    DarkCyan = 36,
    DarkPurple = 90,
    DarkGreen = 22,
    DarkRed = 1,
    DarkBlue = 4,
    DarkOrange = 94,
}


#[derive(PartialEq, Copy, Clone)]
pub enum Direction {
    Left,
    Right
}
