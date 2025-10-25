namespace SpotlightDimmer.Config;

partial class ConfigForm
{
    /// <summary>
    ///  Required designer variable.
    /// </summary>
    private System.ComponentModel.IContainer components = null;

    #region Windows Form Designer generated code

    /// <summary>
    ///  Required method for Designer support - do not modify
    ///  the contents of this method with the code editor.
    /// </summary>
    private void InitializeComponent()
    {
        // Mode ComboBox
        var modeLabel = new Label
        {
            Text = "Dimming Mode:",
            Location = new Point(20, 20),
            Size = new Size(120, 23),
            AutoSize = true
        };

        modeComboBox = new ComboBox
        {
            Location = new Point(20, 45),
            Size = new Size(300, 28),
            DropDownStyle = ComboBoxStyle.DropDownList
        };
        modeComboBox.Items.AddRange(new object[] { "FullScreen", "Partial", "PartialWithActive" });
        modeComboBox.SelectedIndexChanged += OnConfigChanged;

        // Inactive Color Section
        var inactiveColorLabel = new Label
        {
            Text = "Inactive Color:",
            Location = new Point(20, 90),
            Size = new Size(120, 23),
            AutoSize = true
        };

        inactiveColorPanel = new Panel
        {
            Location = new Point(20, 115),
            Size = new Size(50, 50),
            BorderStyle = BorderStyle.FixedSingle
        };
        inactiveColorPanel.Click += (s, e) => SelectColor(inactiveColorPanel, inactiveOpacityNumeric);

        var inactiveOpacityLabel = new Label
        {
            Text = "Opacity:",
            Location = new Point(80, 115),
            Size = new Size(60, 23),
            AutoSize = true
        };

        inactiveOpacityNumeric = new NumericUpDown
        {
            Location = new Point(80, 138),
            Size = new Size(100, 27),
            Minimum = 0,
            Maximum = 255,
            Value = 153
        };
        inactiveOpacityNumeric.ValueChanged += OnConfigChanged;

        // Active Color Section
        var activeColorLabel = new Label
        {
            Text = "Active Color:",
            Location = new Point(20, 190),
            Size = new Size(120, 23),
            AutoSize = true
        };

        activeColorPanel = new Panel
        {
            Location = new Point(20, 215),
            Size = new Size(50, 50),
            BorderStyle = BorderStyle.FixedSingle
        };
        activeColorPanel.Click += (s, e) => SelectColor(activeColorPanel, activeOpacityNumeric);

        var activeOpacityLabel = new Label
        {
            Text = "Opacity:",
            Location = new Point(80, 215),
            Size = new Size(60, 23),
            AutoSize = true
        };

        activeOpacityNumeric = new NumericUpDown
        {
            Location = new Point(80, 238),
            Size = new Size(100, 27),
            Minimum = 0,
            Maximum = 255,
            Value = 102
        };
        activeOpacityNumeric.ValueChanged += OnConfigChanged;

        // Form setup
        AutoScaleMode = AutoScaleMode.Font;
        ClientSize = new Size(350, 300);
        Text = "SpotlightDimmer Configuration";
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        StartPosition = FormStartPosition.CenterScreen;

        // Add controls
        Controls.Add(modeLabel);
        Controls.Add(modeComboBox);
        Controls.Add(inactiveColorLabel);
        Controls.Add(inactiveColorPanel);
        Controls.Add(inactiveOpacityLabel);
        Controls.Add(inactiveOpacityNumeric);
        Controls.Add(activeColorLabel);
        Controls.Add(activeColorPanel);
        Controls.Add(activeOpacityLabel);
        Controls.Add(activeOpacityNumeric);
    }

    #endregion

    private ComboBox modeComboBox = null!;
    private Panel inactiveColorPanel = null!;
    private NumericUpDown inactiveOpacityNumeric = null!;
    private Panel activeColorPanel = null!;
    private NumericUpDown activeOpacityNumeric = null!;
}
