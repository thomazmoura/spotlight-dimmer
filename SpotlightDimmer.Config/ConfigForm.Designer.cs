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

        // Renderer Backend Section
        var rendererLabel = new Label
        {
            Text = "Renderer Backend:",
            Location = new Point(20, 360),
            Size = new Size(120, 23),
            AutoSize = true
        };

        rendererBackendComboBox = new ComboBox
        {
            Location = new Point(20, 385),
            Size = new Size(200, 28),
            DropDownStyle = ComboBoxStyle.DropDownList
        };
        rendererBackendComboBox.Items.AddRange(new object[] { "LayeredWindow", "UpdateLayeredWindow", "CompositeOverlay" });
        rendererBackendComboBox.SelectedIndexChanged += OnRendererBackendChanged;

        var rendererHelpLabel = new Label
        {
            Text = "💡 LayeredWindow: Lightweight (~5 MB)\n   UpdateLayeredWindow: Reduced gaps (~100 MB)\n   CompositeOverlay: Fewer handles (~16 MB)",
            Location = new Point(230, 365),
            Size = new Size(250, 75),
            AutoSize = false,
            ForeColor = System.Drawing.Color.DarkSlateGray
        };

        // Logging Section
        var loggingGroupBox = new GroupBox
        {
            Text = "Logging",
            Location = new Point(20, 435),
            Size = new Size(440, 140)
        };

        enableLoggingCheckBox = new CheckBox
        {
            Text = "Enable Logging",
            Location = new Point(10, 25),
            Size = new Size(150, 24),
            AutoSize = true
        };
        enableLoggingCheckBox.CheckedChanged += OnEnableLoggingChanged;

        var logLevelLabel = new Label
        {
            Text = "Log Level:",
            Location = new Point(10, 55),
            Size = new Size(80, 23),
            AutoSize = true
        };

        logLevelComboBox = new ComboBox
        {
            Location = new Point(100, 52),
            Size = new Size(120, 28),
            DropDownStyle = ComboBoxStyle.DropDownList
        };
        logLevelComboBox.Items.AddRange(new object[] { "Error", "Information", "Debug" });
        logLevelComboBox.SelectedIndexChanged += OnLogLevelChanged;

        var retentionLabel = new Label
        {
            Text = "Retention (days):",
            Location = new Point(10, 90),
            Size = new Size(120, 23),
            AutoSize = true
        };

        logRetentionDaysNumericUpDown = new NumericUpDown
        {
            Location = new Point(135, 87),
            Size = new Size(85, 27),
            Minimum = 1,
            Maximum = 365,
            Value = 7
        };
        logRetentionDaysNumericUpDown.ValueChanged += OnLogRetentionDaysChanged;

        openLogsFolderButton = new Button
        {
            Text = "Open Logs Folder",
            Location = new Point(240, 52),
            Size = new Size(130, 28)
        };
        openLogsFolderButton.Click += OnOpenLogsFolderClicked;

        loggingGroupBox.Controls.Add(enableLoggingCheckBox);
        loggingGroupBox.Controls.Add(logLevelLabel);
        loggingGroupBox.Controls.Add(logLevelComboBox);
        loggingGroupBox.Controls.Add(retentionLabel);
        loggingGroupBox.Controls.Add(logRetentionDaysNumericUpDown);
        loggingGroupBox.Controls.Add(openLogsFolderButton);

        // Experimental Features Section
        var experimentalGroupBox = new GroupBox
        {
            Text = "Experimental Features",
            Location = new Point(20, 585),
            Size = new Size(440, 100)
        };

        excludeFromScreenCaptureCheckBox = new CheckBox
        {
            Text = "Exclude overlays from screenshots",
            Location = new Point(10, 25),
            Size = new Size(420, 24),
            AutoSize = true
        };
        excludeFromScreenCaptureCheckBox.CheckedChanged += OnExcludeFromScreenCaptureChanged;

        var experimentalDisclaimerLabel = new Label
        {
            Text = "⚠ This feature may not work on all Windows systems due to API limitations.\nOverlays will function normally even if exclusion fails.",
            Location = new Point(30, 50),
            Size = new Size(390, 40),
            AutoSize = false,
            ForeColor = System.Drawing.Color.DarkOrange
        };

        experimentalGroupBox.Controls.Add(excludeFromScreenCaptureCheckBox);
        experimentalGroupBox.Controls.Add(experimentalDisclaimerLabel);

        // App Integrations Tab (terminal pane spotlight)
        var integrationsHeaderLabel = new Label
        {
            Text = "Terminal pane spotlight — match a terminal app by process name so the\nspotlight follows the focused pane instead of the whole window.",
            Location = new Point(15, 15),
            Size = new Size(450, 40),
            AutoSize = false,
            ForeColor = System.Drawing.Color.DarkSlateGray
        };

        integrationsListBox = new ListBox
        {
            Location = new Point(15, 60),
            Size = new Size(200, 220)
        };
        integrationsListBox.SelectedIndexChanged += OnIntegrationSelected;

        addIntegrationButton = new Button
        {
            Text = "Add",
            Location = new Point(15, 290),
            Size = new Size(95, 28)
        };
        addIntegrationButton.Click += OnAddIntegration;

        removeIntegrationButton = new Button
        {
            Text = "Remove",
            Location = new Point(120, 290),
            Size = new Size(95, 28),
            Enabled = false
        };
        removeIntegrationButton.Click += OnRemoveIntegration;

        var integrationProcessNameLabel = new Label
        {
            Text = "Process Name:",
            Location = new Point(230, 60),
            Size = new Size(120, 23),
            AutoSize = true
        };

        integrationProcessNameTextBox = new TextBox
        {
            Location = new Point(230, 83),
            Size = new Size(235, 27),
            Enabled = false
        };
        integrationProcessNameTextBox.Leave += OnIntegrationProcessNameLeave;

        var integrationProviderLabel = new Label
        {
            Text = "Provider:",
            Location = new Point(230, 120),
            Size = new Size(120, 23),
            AutoSize = true
        };

        integrationProviderComboBox = new ComboBox
        {
            Location = new Point(230, 143),
            Size = new Size(235, 28),
            DropDownStyle = ComboBoxStyle.DropDownList,
            Enabled = false
        };
        integrationProviderComboBox.Items.AddRange(new object[] { "windows-terminal", "wezterm", "tmux" });
        integrationProviderComboBox.SelectedIndexChanged += OnIntegrationProviderChanged;

        var integrationOffsetXLabel = new Label
        {
            Text = "Content Offset X (px):",
            Location = new Point(230, 181),
            Size = new Size(145, 23),
            AutoSize = true
        };

        integrationOffsetXNumericUpDown = new NumericUpDown
        {
            Location = new Point(380, 178),
            Size = new Size(85, 27),
            Minimum = 0,
            Maximum = 1000,
            Enabled = false
        };
        integrationOffsetXNumericUpDown.ValueChanged += OnIntegrationOffsetChanged;

        var integrationOffsetYLabel = new Label
        {
            Text = "Content Offset Y (px):",
            Location = new Point(230, 214),
            Size = new Size(145, 23),
            AutoSize = true
        };

        integrationOffsetYNumericUpDown = new NumericUpDown
        {
            Location = new Point(380, 211),
            Size = new Size(85, 27),
            Minimum = 0,
            Maximum = 1000,
            Enabled = false
        };
        integrationOffsetYNumericUpDown.ValueChanged += OnIntegrationOffsetChanged;

        var integrationOffsetsHelpLabel = new Label
        {
            Text = "💡 Offsets: pixels from the content area edge to the terminal cell grid\n   (window padding for X, padding + tab bar height for Y).",
            Location = new Point(230, 250),
            Size = new Size(240, 60),
            AutoSize = false,
            ForeColor = System.Drawing.Color.DarkSlateGray
        };

        // Tab control hosting the General and Integrations pages
        var generalTabPage = new TabPage { Text = "General" };
        var integrationsTabPage = new TabPage { Text = "Integrations" };
        mainTabControl = new TabControl { Dock = DockStyle.Fill };
        mainTabControl.TabPages.Add(generalTabPage);
        mainTabControl.TabPages.Add(integrationsTabPage);

        // Form setup
        AutoScaleMode = AutoScaleMode.Font;
        ClientSize = new Size(490, 740);
        Text = "SpotlightDimmer Configuration";
        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = false;
        StartPosition = FormStartPosition.CenterScreen;

        // Add controls to the General tab
        generalTabPage.Controls.Add(profileLabel);
        generalTabPage.Controls.Add(profileComboBox);
        generalTabPage.Controls.Add(saveProfileButton);
        generalTabPage.Controls.Add(deleteProfileButton);
        generalTabPage.Controls.Add(modeLabel);
        generalTabPage.Controls.Add(modeComboBox);
        generalTabPage.Controls.Add(inactiveColorLabel);
        generalTabPage.Controls.Add(inactiveColorPanel);
        generalTabPage.Controls.Add(inactiveOpacityLabel);
        generalTabPage.Controls.Add(inactiveOpacityTrackBar);
        generalTabPage.Controls.Add(inactiveOpacityValueLabel);
        generalTabPage.Controls.Add(activeColorLabel);
        generalTabPage.Controls.Add(activeColorPanel);
        generalTabPage.Controls.Add(activeOpacityLabel);
        generalTabPage.Controls.Add(activeOpacityTrackBar);
        generalTabPage.Controls.Add(activeOpacityValueLabel);
        generalTabPage.Controls.Add(rendererLabel);
        generalTabPage.Controls.Add(rendererBackendComboBox);
        generalTabPage.Controls.Add(rendererHelpLabel);
        generalTabPage.Controls.Add(loggingGroupBox);
        generalTabPage.Controls.Add(experimentalGroupBox);

        // Add controls to the Integrations tab
        integrationsTabPage.Controls.Add(integrationsHeaderLabel);
        integrationsTabPage.Controls.Add(integrationsListBox);
        integrationsTabPage.Controls.Add(addIntegrationButton);
        integrationsTabPage.Controls.Add(removeIntegrationButton);
        integrationsTabPage.Controls.Add(integrationProcessNameLabel);
        integrationsTabPage.Controls.Add(integrationProcessNameTextBox);
        integrationsTabPage.Controls.Add(integrationProviderLabel);
        integrationsTabPage.Controls.Add(integrationProviderComboBox);
        integrationsTabPage.Controls.Add(integrationOffsetXLabel);
        integrationsTabPage.Controls.Add(integrationOffsetXNumericUpDown);
        integrationsTabPage.Controls.Add(integrationOffsetYLabel);
        integrationsTabPage.Controls.Add(integrationOffsetYNumericUpDown);
        integrationsTabPage.Controls.Add(integrationOffsetsHelpLabel);

        Controls.Add(mainTabControl);
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
    private ComboBox rendererBackendComboBox = null!;
    private CheckBox enableLoggingCheckBox = null!;
    private ComboBox logLevelComboBox = null!;
    private NumericUpDown logRetentionDaysNumericUpDown = null!;
    private Button openLogsFolderButton = null!;
    private CheckBox excludeFromScreenCaptureCheckBox = null!;
    private TabControl mainTabControl = null!;
    private ListBox integrationsListBox = null!;
    private Button addIntegrationButton = null!;
    private Button removeIntegrationButton = null!;
    private TextBox integrationProcessNameTextBox = null!;
    private ComboBox integrationProviderComboBox = null!;
    private NumericUpDown integrationOffsetXNumericUpDown = null!;
    private NumericUpDown integrationOffsetYNumericUpDown = null!;
}
