using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Forms;
using System.Windows.Media;

namespace SpotlightDimmer.Models
{
    public class DimmerState: INotifyPropertyChanged
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
                _debugInfo += $"\r\n{value}\r\n______________________\r\n";
                OnPropertyChanged(nameof(DebugInfo));
            }
        }

        private IEnumerable<Screen> _unfocusedScreens = Enumerable.Empty<Screen>();
        public IEnumerable<Screen> UnfocusedScreens
        {
            get => _unfocusedScreens;
            set
            {
                _unfocusedScreens = value;
                OnPropertyChanged(nameof(UnfocusedScreens));
                OnPropertyChanged(nameof(UnfocusedScreenNames));
            }
        }
        public string UnfocusedScreenNames => String.Join(", ", UnfocusedScreens.Select(screen => screen.DeviceName).ToList());

        private bool _isDebugInfoVisible;
        private Color? _selectedColor;
        public Color SelectedColor
        {
            get {
                return _selectedColor?? Color.FromArgb(0, 0, 0, 0);
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
            get {
                return _activeWindowInfo?? new ActiveWindowInfo("Indefinido", new RECT());
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
}
