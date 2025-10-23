using SpotlightDimmer.Core;

namespace SpotlightDimmer;

/// <summary>
/// Test program to verify OverlayCalculator object reuse optimization
/// </summary>
internal class TestOverlayCalculator
{
    public static void Run()
    {
        Console.WriteLine("=== Testing OverlayCalculator Optimizations ===\n");

        var calculator = new OverlayCalculator();

        // Create test displays
        var displays = new[]
        {
            new DisplayInfo(0, new Rectangle(0, 0, 1920, 1080)),
            new DisplayInfo(1, new Rectangle(1920, 0, 1920, 1080))
        };

        var config = new OverlayCalculationConfig(
            DimmingMode.Partial,
            Color.Black,
            153,
            new Color(255, 0, 0), // Red
            102
        );

        Console.WriteLine("Test 1: First calculation (should create new state objects)");
        var states1 = calculator.Calculate(displays, new Rectangle(100, 100, 800, 600), 0, config);
        PrintStates(states1);

        Console.WriteLine("\nTest 2: Second calculation with different window position (should reuse state objects)");
        var states2 = calculator.Calculate(displays, new Rectangle(200, 200, 800, 600), 0, config);
        PrintStates(states2);

        Console.WriteLine("\nTest 3: Verify object reuse (states should be the same instances)");
        Console.WriteLine($"Display 0 reused: {ReferenceEquals(states1[0], states2[0])}");
        Console.WriteLine($"Display 1 reused: {ReferenceEquals(states1[1], states2[1])}");

        Console.WriteLine("\nTest 4: Full-screen mode (no overlays on focused display)");
        var configFullScreen = new OverlayCalculationConfig(
            DimmingMode.FullScreen,
            Color.Black,
            153,
            new Color(255, 0, 0), // Red
            102
        );
        var states3 = calculator.Calculate(displays, new Rectangle(100, 100, 800, 600), 0, configFullScreen);
        PrintStates(states3);

        Console.WriteLine("\nTest 5: PartialWithActive mode");
        var configWithActive = new OverlayCalculationConfig(
            DimmingMode.PartialWithActive,
            Color.Black,
            153,
            new Color(255, 0, 0), // Red
            102
        );
        var states4 = calculator.Calculate(displays, new Rectangle(100, 100, 800, 600), 0, configWithActive);
        PrintStates(states4);

        Console.WriteLine("\n=== All tests completed successfully! ===");
    }

    private static void PrintStates(DisplayOverlayState[] states)
    {
        foreach (var state in states)
        {
            Console.WriteLine($"Display {state.DisplayIndex} ({state.VisibleOverlayCount} visible overlays):");
            foreach (var overlay in state.Overlays)
            {
                if (overlay.IsVisible)
                {
                    Console.WriteLine($"  - {overlay.Region}: {overlay.Bounds} (opacity: {overlay.Opacity})");
                }
            }
        }
    }
}
