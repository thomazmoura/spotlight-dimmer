using SpotlightDimmer.Core;

namespace SpotlightDimmer;

/// <summary>
/// Test program to verify AppState in-place update optimization
/// </summary>
internal class TestOverlayCalculator
{
    public static void Run()
    {
        Console.WriteLine("=== Testing AppState In-Place Updates ===\n");

        // Create test displays
        var displays = new[]
        {
            new DisplayInfo(0, new Rectangle(0, 0, 1920, 1080)),
            new DisplayInfo(1, new Rectangle(1920, 0, 1920, 1080))
        };

        // Create AppState with pre-allocated overlay definitions
        var appState = new AppState(displays);

        var config = new OverlayCalculationConfig(
            DimmingMode.Partial,
            Color.Black,
            153,
            new Color(255, 0, 0), // Red
            102
        );

        Console.WriteLine("Test 1: First calculation (updates pre-allocated state objects)");
        appState.Calculate(displays, new Rectangle(100, 100, 800, 600), 0, config);
        var states1 = appState.DisplayStates;
        PrintStates(states1);

        Console.WriteLine("\nTest 2: Second calculation with different window position (reuses same state objects)");
        appState.Calculate(displays, new Rectangle(200, 200, 800, 600), 0, config);
        var states2 = appState.DisplayStates;
        PrintStates(states2);

        Console.WriteLine("\nTest 3: Verify object reuse (states array should be the same instance)");
        Console.WriteLine($"DisplayStates array reused: {ReferenceEquals(states1, states2)}");
        Console.WriteLine($"Display 0 reused: {ReferenceEquals(states1[0], states2[0])}");
        Console.WriteLine($"Display 1 reused: {ReferenceEquals(states1[1], states2[1])}");
        Console.WriteLine($"Display 0 Overlay 0 reused: {ReferenceEquals(states1[0].Overlays[0], states2[0].Overlays[0])}");

        Console.WriteLine("\nTest 4: Full-screen mode (no overlays on focused display)");
        var configFullScreen = new OverlayCalculationConfig(
            DimmingMode.FullScreen,
            Color.Black,
            153,
            new Color(255, 0, 0), // Red
            102
        );
        appState.Calculate(displays, new Rectangle(100, 100, 800, 600), 0, configFullScreen);
        PrintStates(appState.DisplayStates);

        Console.WriteLine("\nTest 5: PartialWithActive mode");
        var configWithActive = new OverlayCalculationConfig(
            DimmingMode.PartialWithActive,
            Color.Black,
            153,
            new Color(255, 0, 0), // Red
            102
        );
        appState.Calculate(displays, new Rectangle(100, 100, 800, 600), 0, configWithActive);
        PrintStates(appState.DisplayStates);

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
