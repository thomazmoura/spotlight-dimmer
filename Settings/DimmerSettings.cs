
using System.ComponentModel;
using System.Windows;
using System.Windows.Media;

namespace SpotlightDimmer.Settings
{
    public class DimmerSettings: INotifyPropertyChanged
    {
        public DimmerSettings()
        {
            IsDebugInfoVisible = true;
            SelectedColor = Color.FromArgb(128, 64, 64, 128);
        }

        private bool _isDebugInfoVisible;
        private Color _selectedColor;
        public Color SelectedColor
        {
            get { return _selectedColor; }
            set
            {
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

        private string _debugInfo = "This is some debug information.";

        public string DebugInfo
        {
            get { return _debugInfo; }
            set
            {
                _debugInfo = value;
                OnPropertyChanged(nameof(DebugInfo));
            }
        }
        public Visibility DebugInfoVisibility => IsDebugInfoVisible ? Visibility.Visible : Visibility.Collapsed;

        public event PropertyChangedEventHandler PropertyChanged;

        protected void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
