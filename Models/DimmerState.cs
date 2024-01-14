using Brush = System.Windows.Media.Brush;
using Color = System.Windows.Media.Color;

namespace SpotlightDimmer.Models;

public class DimmerState : INotifyPropertyChanged
{
    public event PropertyChangedEventHandler? PropertyChanged;
    public void OnPropertyChanged(string propertyName)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }

    private string _debugInfo = "";
    public string DebugInfo
    {
        get { return _debugInfo; }
        set
        {
            _debugInfo += $"\r\n\r\n{value}";
            OnPropertyChanged(nameof(DebugInfo));
        }
    }


    private bool _verbose = false;
    public bool Verbose
    {
        get { return _verbose; }
        set
        {
            _verbose = value;
            OnPropertyChanged(nameof(Verbose));
        }
    }

    private bool _topMost = false;
    public bool Topmost
    {
        get { return _topMost; }
        set
        {
            _topMost = value;
            OnPropertyChanged(nameof(Topmost));
        }
    }

    private bool _minimizeToTray = false;
    public bool MinimizeToTray
    {
        get { return _minimizeToTray; }
        set
        {
            _minimizeToTray = value;
            OnPropertyChanged(nameof(MinimizeToTray));

        }
    }

    private bool _startWithWindows = false;



    private bool _startMinimized = false;
    public bool StartMinimized
    {
        get { return _startMinimized; }
        set
        {
            _startMinimized = value;
            OnPropertyChanged(nameof(StartMinimized));
        }
    }

    private Screen _focusedScreen = Screen.PrimaryScreen;
    public Screen FocusedScreen
    {
        get => _focusedScreen;
        set
        {
            _focusedScreen = value;
            OnPropertyChanged(nameof(FocusedScreen));
            OnPropertyChanged(nameof(FocusedScreenName));
        }
    }
    public string FocusedScreenName => FocusedScreen.DeviceName;

    private bool _isDebugInfoVisible;
    private Color? _selectedColor;
    public Color SelectedColor
    {
        get
        {
            return _selectedColor ?? Color.FromArgb(0, 0, 0, 0);
        }
        set
        {
            if (value.A > 225)
                value.A = 225;
            _selectedColor = value;
            OnPropertyChanged(nameof(SelectedColor));
            OnPropertyChanged(nameof(SelectedBrush));
        }
    }
    public Brush SelectedBrush
    {
        get { return new SolidColorBrush(SelectedColor); }
    }
    public bool IsDebugInfoVisible
    {
        get { return _isDebugInfoVisible; }
        set
        {
            _isDebugInfoVisible = value;
            OnPropertyChanged(nameof(DebugInfoVisibility));
        }
    }
    public Visibility DebugInfoVisibility => IsDebugInfoVisible ? Visibility.Visible : Visibility.Collapsed;

    private ActiveWindowInfo? _activeWindowInfo;
    public ActiveWindowInfo ActiveWindowInfo
    {
        get
        {
            return _activeWindowInfo ?? new ActiveWindowInfo("Indefinido", new RECT());
        }
        set
        {
            _activeWindowInfo = value;
            OnPropertyChanged(nameof(ActiveWindowInfo));
        }
    }
}

public record ActiveWindowInfo(string Title, RECT BoundsRectangle)
{
    public override string ToString()
    {
        return $"Title: {Title}\r\n\r\nBounds:\r\nLeft:{BoundsRectangle.left}\r\nTop:{BoundsRectangle.top}\r\nRight:{BoundsRectangle.right}\r\nBottom:{BoundsRectangle.bottom}";
    }
}
