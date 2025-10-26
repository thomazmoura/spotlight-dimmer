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
        // Profile Section
        var profileLabel = new Label
        {
            Text = "Profile:",
            Location = new Point(20, 20),
            Size = new Size(120, 23),
            AutoSize = true
        };

        profileComboBox = new ComboBox
        {
            Location = new Point(20, 45),
            Size = new Size(200, 28),
            DropDownStyle = ComboBoxStyle.DropDownList
        };
        profileComboBox.SelectedIndexChanged += OnProfileSelected;

        saveProfileButton = new Button
        {
            Text = "Save Profile...",
            Location = new Point(230, 44),
            Size = new Size(110, 28)
        };
        saveProfileButton.Click += OnSaveProfile;

        deleteProfileButton = new Button
        {
            Text = "Delete Profile",
            Location = new Point(350, 44),
            Size = new Size(110, 28),
            Enabled = false
        };
        deleteProfileButton.Click += OnDeleteProfile;

        // Mode ComboBox
        var modeLabel = new Label
        {
            Text = "Dimming Mode:",
            Location = new Point(20, 90),
            Size = new Size(120, 23),
            AutoSize = true
        };

        modeComboBox = new ComboBox
        {
            Location = new Point(20, 115),
            Size = new Size(300, 28),
            DropDownStyle = ComboBoxStyle.DropDownList
        };
        modeComboBox.Items.AddRange(new object[] { "FullScreen", "Partial", "PartialWithActive" });
        modeComboBox.SelectedIndexChanged += OnConfigChanged;

        // Inactive Color Section
        var inactiveColorLabel = new Label
        {
            Text = "Inactive Color:",
            Location = new Point(20, 160),
            Size = new Size(120, 23),
            AutoSize = true
        };

        inactiveColorPanel = new Panel
        {
            Location = new Point(20, 185),
            Size = new Size(50, 50),
            BorderStyle = BorderStyle.FixedSingle
        };
        inactiveColorPanel.Click += (s, e) => SelectColor(inactiveColorPanel, inactiveOpacityTrackBar);

        var inactiveOpacityLabel = new Label
        {
            Text = "Opacity:",
            Location = new Point(80, 185),
            Size = new Size(60, 23),
            AutoSize = true
        };

        inactiveOpacityTrackBar = new TrackBar
        {
            Location = new Point(80, 208),
            Size = new Size(180, 45),
            Minimum = 0,
            Maximum = 255,
            TickFrequency = 25,
            Value = 153
        };
        inactiveOpacityTrackBar.ValueChanged += OnOpacityChanged;

        inactiveOpacityValueLabel = new Label
        {
            Text = "153",
            Location = new Point(270, 208),
            Size = new Size(50, 23),
            AutoSize = true
        };

        // Active Color Section
        var activeColorLabel = new Label
        {
            Text = "Active Color:",
            Location = new Point(20, 260),
            Size = new Size(120, 23),
            AutoSize = true
        };

        activeColorPanel = new Panel
        {
            Location = new Point(20, 285),
            Size = new Size(50, 50),
            BorderStyle = BorderStyle.FixedSingle
        };
        activeColorPanel.Click += (s, e) => SelectColor(activeColorPanel, activeOpacityTrackBar);

        var activeOpacityLabel = new Label
        {
            Text = "Opacity:",
            Location = new Point(80, 285),
            Size = new Size(60, 23),
            AutoSize = true
        };

        activeOpacityTrackBar = new TrackBar
        {
            Location = new Point(80, 308),
            Size = new Size(180, 45),
            Minimum = 0,
            Maximum = 255,
            TickFrequency = 25,
            Value = 102
        };
        activeOpacityTrackBar.ValueChanged += OnOpacityChanged;

        activeOpacityValueLabel = new Label
        {
            Text = "102",
            Location = new Point(270, 308),
            Size = new Size(50, 23),
            AutoSize = true
        };

        // Verbose Logging Section
        verboseLoggingCheckBox = new CheckBox
        {
            Text = "Verbose Logging",
            Location = new Point(20, 365),
            Size = new Size(150, 24),
            AutoSize = true
        };
        verboseLoggingCheckBox.CheckedChanged += OnVerboseLoggingChanged;

        // Form setup
        AutoScaleMode = AutoScaleMode.Font;
        ClientSize = new Size(480, 410);
        Text = "SpotlightDimmer Configuration";
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        StartPosition = FormStartPosition.CenterScreen;

        // Add controls
        Controls.Add(profileLabel);
        Controls.Add(profileComboBox);
        Controls.Add(saveProfileButton);
        Controls.Add(deleteProfileButton);
        Controls.Add(modeLabel);
        Controls.Add(modeComboBox);
        Controls.Add(inactiveColorLabel);
        Controls.Add(inactiveColorPanel);
        Controls.Add(inactiveOpacityLabel);
        Controls.Add(inactiveOpacityTrackBar);
        Controls.Add(inactiveOpacityValueLabel);
        Controls.Add(activeColorLabel);
        Controls.Add(activeColorPanel);
        Controls.Add(activeOpacityLabel);
        Controls.Add(activeOpacityTrackBar);
        Controls.Add(activeOpacityValueLabel);
        Controls.Add(verboseLoggingCheckBox);
    }

    #endregion

    private ComboBox profileComboBox = null!;
    private Button saveProfileButton = null!;
    private Button deleteProfileButton = null!;
    private ComboBox modeComboBox = null!;
    private Panel inactiveColorPanel = null!;
    private TrackBar inactiveOpacityTrackBar = null!;
    private Label inactiveOpacityValueLabel = null!;
    private Panel activeColorPanel = null!;
    private TrackBar activeOpacityTrackBar = null!;
    private Label activeOpacityValueLabel = null!;
    private CheckBox verboseLoggingCheckBox = null!;
}
