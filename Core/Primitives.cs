namespace SpotlightDimmer.Core;

/// <summary>
/// Lightweight rectangle structure for AOT-friendly geometry calculations.
/// Represents a rectangular region with position and size.
/// </summary>
public readonly record struct Rectangle(int X, int Y, int Width, int Height)
{
    /// <summary>
    /// Gets the x-coordinate of the left edge.
    /// </summary>
    public int Left => X;

    /// <summary>
    /// Gets the y-coordinate of the top edge.
    /// </summary>
    public int Top => Y;

    /// <summary>
    /// Gets the x-coordinate of the right edge.
    /// </summary>
    public int Right => X + Width;

    /// <summary>
    /// Gets the y-coordinate of the bottom edge.
    /// </summary>
    public int Bottom => Y + Height;

    /// <summary>
    /// Creates a rectangle from left, top, right, bottom coordinates.
    /// </summary>
    public static Rectangle FromLTRB(int left, int top, int right, int bottom)
    {
        return new Rectangle(left, top, right - left, bottom - top);
    }

    /// <summary>
    /// Checks if this rectangle contains the specified point.
    /// </summary>
    public bool Contains(int x, int y)
    {
        return x >= X && x < X + Width && y >= Y && y < Y + Height;
    }

    /// <summary>
    /// Checks if this rectangle intersects with another rectangle.
    /// </summary>
    public bool IntersectsWith(Rectangle other)
    {
        return Left < other.Right && Right > other.Left &&
               Top < other.Bottom && Bottom > other.Top;
    }
}

/// <summary>
/// Lightweight RGB color structure for AOT-friendly color representation.
/// Opacity is handled separately in overlay definitions.
/// </summary>
public readonly record struct Color(byte R, byte G, byte B)
{
    /// <summary>
    /// Black color (0, 0, 0).
    /// </summary>
    public static readonly Color Black = new(0, 0, 0);

    /// <summary>
    /// White color (255, 255, 255).
    /// </summary>
    public static readonly Color White = new(255, 255, 255);

    /// <summary>
    /// Creates a color from a 32-bit RGB value.
    /// </summary>
    public static Color FromRgb(uint rgb)
    {
        return new Color(
            (byte)((rgb >> 16) & 0xFF),
            (byte)((rgb >> 8) & 0xFF),
            (byte)(rgb & 0xFF)
        );
    }

    /// <summary>
    /// Converts this color to a 32-bit RGB value.
    /// </summary>
    public uint ToRgb()
    {
        return ((uint)R << 16) | ((uint)G << 8) | B;
    }
}
