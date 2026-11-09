pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls.Basic
import Quickshell
import Quickshell.Io
import Quickshell.Services.Pipewire
import Quickshell.Services.SystemTray
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// The Control Panel: a normal window, so it gets the Aero frame, laid out as
// the Windows 7 one: an address bar with the path, a blue pane of links on
// the left, and the page on the right. The home page lists the categories;
// each link opens a page of settings, which apply at once.
//
// Everything here is kept by Settings.qml, except the light or dark choice,
// which is the system's own and is read by every program.
FloatingWindow {
    id: window

    title: "Control Panel"
    visible: ControlPanelState.shown
    color: Scheme.page
    implicitWidth: 860
    implicitHeight: 600
    minimumSize: Qt.size(640, 420)

    // The frame's close button.
    onVisibleChanged: {
        if (!visible)
            ControlPanelState.close();
    }

    readonly property string page: ControlPanelState.page

    readonly property var pages: ({
        "home": { title: "Control Panel Home", category: "" },
        "appearance": { title: "Light, dark and glass", category: "Appearance and Personalization" },
        "background": { title: "Desktop Background", category: "Appearance and Personalization" },
        "lockscreen": { title: "Lock Screen", category: "Appearance and Personalization" },
        "account": { title: "Account Picture", category: "User Accounts" },
        "taskbar": { title: "Taskbar and Start Menu", category: "Appearance and Personalization" },
        "notifications": { title: "Notification Area Icons", category: "Appearance and Personalization" },
        "clock": { title: "Date and Time", category: "Clock and Region" },
        "ease": { title: "Ease of Access", category: "Ease of Access" },
        "startup": { title: "Startup Programs", category: "System" },
        "defaults": { title: "Default Programs", category: "Programs" },
        "sound": { title: "Sound", category: "Hardware and Sound" },
        "devices": { title: "Devices and Printers", category: "Hardware and Sound" },
        "display": { title: "Screen Resolution", category: "Appearance and Personalization" },
        "about": { title: "System", category: "System" }
    })

    // ----- search -------------------------------------------------------------
    //
    // Every task the pages hold, with the words it is found by. Typing in the
    // search box replaces the page with the tasks that match every word.

    readonly property var tasks: ControlPanelState.tasks

    property string query: ""

    readonly property var found: {
        var words = window.query.toLowerCase().split(/\s+/).filter(function (word) {
            return word.length > 0;
        });
        if (words.length === 0)
            return [];
        return window.tasks.filter(function (task) {
            var haystack = (task.text + " " + task.words + " "
                + window.pages[task.page].title + " " + window.pages[task.page].category).toLowerCase();
            return words.every(function (word) { return haystack.indexOf(word) >= 0; });
        });
    }

    function go(page) {
        window.query = "";
        searchField.text = "";
        ControlPanelState.page = page;
        flick.contentY = 0;
    }

    // ----- the address bar --------------------------------------------------

    Rectangle {
        id: bar

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 40
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.sideTop }
            GradientStop { position: 1.0; color: Scheme.sideBottom }
        }

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: Scheme.menuDivider
        }

        // Back, as the round arrow of Explorer.
        Rectangle {
            id: back

            anchors.left: parent.left
            anchors.leftMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            width: 26
            height: 26
            radius: 13
            opacity: window.page === "home" ? 0.45 : 1
            border.width: 1
            border.color: "#1d4f86"
            gradient: Gradient {
                GradientStop { position: 0.0; color: backArea.containsMouse ? "#7cc0ff" : "#5aa9f0" }
                GradientStop { position: 1.0; color: "#1c63ae" }
            }

            Image {
                anchors.centerIn: parent
                width: 14
                height: 14
                rotation: 180
                source: "../assets/arrow-right.svg"
                sourceSize.width: 28
                sourceSize.height: 28
            }

            MouseArea {
                id: backArea
                anchors.fill: parent
                hoverEnabled: true
                enabled: window.page !== "home"
                onClicked: window.go("home")
            }
        }

        // "Search Control Panel", at the right.
        Rectangle {
            id: searchBox

            anchors.right: parent.right
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            width: 210
            height: 24
            radius: Scheme.fieldRadius
            color: Scheme.field
            border.width: 1
            border.color: searchField.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

            TextInput {
                id: searchField

                anchors.left: parent.left
                anchors.right: searchIcon.left
                anchors.leftMargin: 6
                anchors.rightMargin: 4
                anchors.verticalCenter: parent.verticalCenter
                color: Scheme.text
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
                selectByMouse: true
                clip: true
                onTextChanged: {
                    window.query = text;
                    flick.contentY = 0;
                }
                Keys.onEscapePressed: text = ""
                Keys.onReturnPressed: {
                    if (window.found.length > 0)
                        window.go(window.found[0].page);
                }

                Text {
                    anchors.fill: parent
                    verticalAlignment: Text.AlignVCenter
                    text: "Search Control Panel"
                    color: Scheme.placeholder
                    font: searchField.font
                    elide: Text.ElideRight
                    visible: searchField.text.length === 0
                }
            }

            Image {
                id: searchIcon
                anchors.right: parent.right
                anchors.rightMargin: 6
                anchors.verticalCenter: parent.verticalCenter
                width: 14
                height: 14
                source: "../assets/search.svg"
                sourceSize.width: 28
                sourceSize.height: 28
            }
        }

        // The path: Control Panel > category > page.
        Rectangle {
            anchors.left: back.right
            anchors.leftMargin: 8
            anchors.right: searchBox.left
            anchors.rightMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            height: 24
            radius: Scheme.fieldRadius
            color: Scheme.field
            border.width: 1
            border.color: Scheme.fieldBorder

            Row {
                anchors.left: parent.left
                anchors.leftMargin: 6
                anchors.verticalCenter: parent.verticalCenter
                spacing: 6

                Image {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 16
                    height: 16
                    source: "../assets/icons/control-panel.png"
                    sourceSize.width: 32
                    sourceSize.height: 32
                }

                CpLink {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Control Panel"
                    color: Scheme.text
                    onClicked: window.go("home")
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: window.page !== "home"
                    text: "▸  " + (window.pages[window.page] || {}).category
                        + "  ▸  " + (window.pages[window.page] || {}).title
                    color: Scheme.text
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }
            }
        }
    }

    // ----- the pane of links --------------------------------------------------

    Rectangle {
        id: side

        anchors.left: parent.left
        anchors.top: bar.bottom
        anchors.bottom: parent.bottom
        width: 210
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.sideTop }
            GradientStop { position: 1.0; color: Scheme.sideBottom }
        }

        Column {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 16
            spacing: 9

            CpLink {
                width: parent.width
                text: "Control Panel Home"
                font.bold: window.page === "home"
                onClicked: window.go("home")
            }

            Item { width: 1; height: 6 }

            Repeater {
                model: ["appearance", "background", "display", "lockscreen", "account",
                        "taskbar", "notifications", "clock", "ease", "devices", "sound", "defaults", "startup", "about"]

                delegate: CpLink {
                    required property string modelData
                    width: parent.width
                    text: window.pages[modelData].title
                    font.bold: window.page === modelData
                    onClicked: window.go(modelData)
                }
            }
        }

        Rectangle {
            anchors.right: parent.right
            width: 1
            height: parent.height
            color: Scheme.menuDivider
        }
    }

    // ----- the page -----------------------------------------------------------

    Flickable {
        id: flick

        anchors.left: side.right
        anchors.right: parent.right
        anchors.top: bar.bottom
        anchors.bottom: parent.bottom
        contentHeight: content.implicitHeight + 48
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: AeroScrollBar { id: pageBar }

        Loader {
            id: content

            x: 28
            y: 22
            width: flick.width - 56 - (pageBar.visible ? pageBar.width : 0)
            sourceComponent: {
                if (window.query.trim().length > 0)
                    return searchPage;
                switch (window.page) {
                case "appearance": return appearancePage;
                case "background": return backgroundPage;
                case "lockscreen": return lockPage;
                case "account": return accountPage;
                case "taskbar": return taskbarPage;
                case "notifications": return notificationsPage;
                case "clock": return clockPage;
                case "ease": return easePage;
                case "startup": return startupPage;
                case "defaults": return defaultsPage;
                case "sound": return soundPage;
                case "devices": return devicesPage;
                case "display": return displayPage;
                case "about": return aboutPage;
                default: return homePage;
                }
            }
        }
    }

    // ----- the pages ----------------------------------------------------------

    component Heading: Text {
        color: Scheme.pageHeading
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize + 8
        wrapMode: Text.WordWrap
    }

    component SubHeading: Text {
        color: Scheme.pageHeading
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize + 2
        topPadding: 10
    }

    component Note: Text {
        color: Scheme.textMuted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        wrapMode: Text.WordWrap
    }

    // A category of the home page: its icon, its name, its links.
    component Category: Row {
        id: category

        property string icon: ""
        property string name: ""
        property string target: ""
        property var links: []

        spacing: 14

        Image {
            width: 48
            height: 48
            source: "../assets/icons/" + category.icon + ".png"
            sourceSize.width: 96
            sourceSize.height: 96
        }

        Column {
            spacing: 4

            CpLink {
                text: category.name
                large: true
                font.bold: true
                onClicked: window.go(category.target)
            }

            Repeater {
                model: category.links

                delegate: CpLink {
                    required property var modelData
                    text: modelData.text
                    onClicked: window.go(modelData.page)
                }
            }
        }
    }

    Component {
        id: searchPage

        Column {
            spacing: 10

            Heading {
                text: window.found.length > 0
                    ? "Results for “" + window.query.trim() + "”"
                    : "No setting matches “" + window.query.trim() + "”"
            }

            Repeater {
                model: window.found

                delegate: Column {
                    required property var modelData

                    spacing: 2

                    CpLink {
                        text: modelData.text
                        large: true
                        onClicked: window.go(modelData.page)
                    }

                    Note {
                        text: window.pages[modelData.page].category
                            + "  ▸  " + window.pages[modelData.page].title
                    }
                }
            }
        }
    }

    Component {
        id: easePage

        Column {
            spacing: 10

            Heading { text: "Make the computer easier to see" }

            CpCheck {
                label: "Turn off all unnecessary animations (when possible)"
                checked: !Settings.animations
                onToggled: function (on) { Settings.setAnimations(!on); }
            }

            Note {
                width: parent.width
                text: "This is the system's own setting: GTK, libadwaita and the "
                    + "programs that read the settings portal follow it too. The "
                    + "shell's fades and the windows' animations stop at once."
            }

            SubHeading { text: "Animation speed" }

            Note {
                width: parent.width
                text: "How fast windows open, close, minimise and snap, and how fast "
                    + "the menus fade. 1x is the speed of Windows 7."
            }

            Row {
                spacing: 12
                enabled: Settings.animations
                opacity: enabled ? 1 : 0.5

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Slower"
                    color: Scheme.textMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }

                // A track from 0.25x to 3x in steps of 0.25x.
                Item {
                    id: speedSlider

                    readonly property real low: 0.25
                    readonly property real high: 3
                    readonly property real fraction:
                        (Settings.animationSpeed - low) / (high - low)

                    anchors.verticalCenter: parent.verticalCenter
                    width: 260
                    height: 24

                    function setFrom(x) {
                        var value = low + Math.max(0, Math.min(1, x / width)) * (high - low);
                        Settings.animationSpeed = Math.round(value * 4) / 4;
                    }

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width
                        height: 4
                        radius: 2
                        color: Scheme.field
                        border.width: 1
                        border.color: Scheme.fieldBorder
                    }

                    // A tick at 1x.
                    Rectangle {
                        x: (1 - speedSlider.low) / (speedSlider.high - speedSlider.low)
                            * speedSlider.width - 0.5
                        y: parent.height - 5
                        width: 1
                        height: 5
                        color: Scheme.fieldBorder
                    }

                    Rectangle {
                        x: speedSlider.fraction * speedSlider.width - width / 2
                        anchors.verticalCenter: parent.verticalCenter
                        width: 10
                        height: 20
                        radius: 2
                        border.width: 1
                        border.color: speedArea.containsMouse || speedArea.pressed
                            ? Scheme.fieldFocus : Scheme.fieldBorder
                        gradient: Gradient {
                            GradientStop { position: 0.0; color: speedArea.containsMouse ? Scheme.buttonHotTop : Scheme.buttonTop }
                            GradientStop { position: 1.0; color: speedArea.containsMouse ? Scheme.buttonHotBottom : Scheme.buttonBottom }
                        }
                    }

                    MouseArea {
                        id: speedArea
                        anchors.fill: parent
                        anchors.leftMargin: -6
                        anchors.rightMargin: -6
                        hoverEnabled: true
                        preventStealing: true
                        onPressed: function (event) { speedSlider.setFrom(event.x - 6); }
                        onPositionChanged: function (event) {
                            if (pressed)
                                speedSlider.setFrom(event.x - 6);
                        }
                    }
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Faster"
                    color: Scheme.textMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: Settings.animationSpeed.toFixed(2) + "x"
                    color: Scheme.text
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                    font.bold: true
                }

                CpLink {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: Settings.animationSpeed !== 1
                    text: "Reset"
                    onClicked: Settings.animationSpeed = 1
                }
            }
        }
    }

    Component {
        id: displayPage

        Column {
            id: display

            /// The screen being set, by connector name.
            property string selected: ""
            readonly property var screen: {
                var list = DisplaySettings.screens;
                for (var i = 0; i < list.length; i++) {
                    if (list[i].name === display.selected)
                        return list[i];
                }
                return list.length > 0 ? list[0] : null;
            }
            /// The screen's resolutions, largest first, and the refresh
            /// rates of the one shown.
            readonly property var sizes: {
                var out = [];
                var modes = display.screen ? display.screen.modes : [];
                for (var i = 0; i < modes.length; i++) {
                    var label = modes[i].width + " × " + modes[i].height;
                    if (out.indexOf(label) < 0)
                        out.push(label);
                }
                return out;
            }
            property int sizeIndex: 0
            readonly property var rates: {
                var out = [];
                var modes = display.screen ? display.screen.modes : [];
                var size = display.sizes[display.sizeIndex] || "";
                for (var i = 0; i < modes.length; i++) {
                    if (modes[i].width + " × " + modes[i].height === size
                        && out.indexOf(modes[i].refreshRate) < 0)
                        out.push(modes[i].refreshRate);
                }
                return out;
            }
            property int rateIndex: 0
            readonly property var scales: [1, 1.25, 1.5, 1.75, 1.875, 2, 2.25, 2.5, 3]
            property int scaleIndex: 0

            /// Show the selected screen's current mode and scale.
            function load() {
                var screen = display.screen;
                if (!screen)
                    return;
                display.sizeIndex = Math.max(0, display.sizes.indexOf(screen.width + " × " + screen.height));
                display.rateIndex = Math.max(0, display.rates.indexOf(screen.refreshRate));
                var scale = display.scales.indexOf(screen.scale);
                display.scaleIndex = scale >= 0 ? scale : 0;
            }

            function apply() {
                var screen = display.screen;
                var size = (display.sizes[display.sizeIndex] || "").split(" × ");
                if (!screen || size.length !== 2)
                    return;
                var choices = Object.assign({}, Settings.displayChoices);
                choices[screen.name] = {
                    width: Number(size[0]),
                    height: Number(size[1]),
                    refreshRate: display.rates[display.rateIndex] || 0,
                    scale: display.scales[display.scaleIndex]
                };
                DisplaySettings.apply(choices);
            }

            spacing: 12

            Component.onCompleted: DisplaySettings.refresh()
            onScreenChanged: display.load()

            Heading { text: "Change the appearance of your displays" }

            // The screens as they are placed, to pick one: a picture of the
            // layout, scaled to fit.
            Rectangle {
                id: layoutFrame

                width: parent.width
                height: 150
                radius: 3
                color: Scheme.dark ? "#141b22" : "#f3f6f9"
                border.width: 1
                border.color: Scheme.fieldBorder

                Item {
                    id: layoutArea

                    readonly property var bounds: {
                        var list = DisplaySettings.screens;
                        var right = 1;
                        var bottom = 1;
                        for (var i = 0; i < list.length; i++) {
                            right = Math.max(right, list[i].x + list[i].width / list[i].scale);
                            bottom = Math.max(bottom, list[i].y + list[i].height / list[i].scale);
                        }
                        return { width: right, height: bottom };
                    }
                    readonly property real k: Math.min((layoutFrame.width - 40) / bounds.width,
                                                       (layoutFrame.height - 30) / bounds.height)

                    width: bounds.width * k
                    height: bounds.height * k
                    anchors.centerIn: parent

                    Repeater {
                        model: DisplaySettings.screens

                        delegate: Rectangle {
                            id: tile

                            required property var modelData
                            required property int index
                            readonly property bool chosen: !!display.screen
                                && tile.modelData.name === display.screen.name

                            x: tile.modelData.x * layoutArea.k + 2
                            y: tile.modelData.y * layoutArea.k + 2
                            width: tile.modelData.width / tile.modelData.scale * layoutArea.k - 4
                            height: tile.modelData.height / tile.modelData.scale * layoutArea.k - 4
                            radius: 2
                            border.width: tile.chosen ? 2 : 1
                            border.color: tile.chosen ? Scheme.fieldFocus : "#5d7489"
                            gradient: Gradient {
                                GradientStop { position: 0.0; color: "#5f9bd1" }
                                GradientStop { position: 1.0; color: "#2d6aa8" }
                            }

                            Text {
                                anchors.centerIn: parent
                                text: tile.index + 1
                                color: "#ffffff"
                                style: Text.Outline
                                styleColor: "#40000000"
                                font.family: Theme.fontFamily
                                font.pixelSize: Math.min(36, parent.height * 0.5)
                                font.bold: true
                            }

                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    display.selected = tile.modelData.name;
                                    display.load();
                                }
                            }
                        }
                    }
                }
            }

            Note {
                width: parent.width
                visible: DisplaySettings.screens.length === 0
                text: "The compositor did not list the screens. This page needs the "
                    + "shell started by the Windows 7 configuration."
            }

            Grid {
                columns: 2
                columnSpacing: 12
                rowSpacing: 10
                visible: !!display.screen
                // The open lists draw over the rows below.
                z: 5

                Note { text: "Display:"; height: 24; verticalAlignment: Text.AlignVCenter }
                CpCombo {
                    width: 300
                    z: open ? 10 : 4
                    model: DisplaySettings.screens.map(function (screen, index) {
                        return (index + 1) + ". " + screen.description + " (" + screen.name + ")";
                    })
                    currentIndex: Math.max(0, DisplaySettings.screens.indexOf(display.screen))
                    onActivated: function (index) {
                        display.selected = DisplaySettings.screens[index].name;
                        display.load();
                    }
                }

                Note { text: "Resolution:"; height: 24; verticalAlignment: Text.AlignVCenter }
                CpCombo {
                    width: 300
                    z: open ? 10 : 3
                    model: display.sizes
                    currentIndex: display.sizeIndex
                    onActivated: function (index) {
                        display.sizeIndex = index;
                        display.rateIndex = 0;
                    }
                }

                Note { text: "Refresh rate:"; height: 24; verticalAlignment: Text.AlignVCenter }
                CpCombo {
                    width: 300
                    z: open ? 10 : 2
                    model: display.rates.map(function (rate) { return rate + " Hz"; })
                    currentIndex: display.rateIndex
                    onActivated: function (index) { display.rateIndex = index; }
                }

                Note { text: "Scale:"; height: 24; verticalAlignment: Text.AlignVCenter }
                CpCombo {
                    width: 300
                    z: open ? 10 : 1
                    model: display.scales.map(function (scale) {
                        return Math.round(scale * 1000) / 10 + " %"
                            + (scale === 1 ? " (smaller)" : scale >= 2 ? " (larger)" : "");
                    })
                    currentIndex: display.scaleIndex
                    onActivated: function (index) { display.scaleIndex = index; }
                }
            }

            // As Windows 7's: where the lock screen, the gadgets and what has
            // no screen of its own go (Screens).
            CpCheck {
                label: "Make this my main display"
                checked: !!display.screen && Screens.primaryName === display.screen.name
                onToggled: function (on) {
                    if (display.screen)
                        Settings.primaryScreen = on ? display.screen.name : "";
                }
            }

            Row {
                spacing: 10
                topPadding: 4
                visible: !DisplaySettings.confirming

                CpButton {
                    label: "Apply"
                    onClicked: display.apply()
                }

                CpButton {
                    label: "Use the session's layout"
                    visible: Object.keys(Settings.displayChoices).length > 0
                    onClicked: DisplaySettings.reset()
                }
            }

            // As on Windows: keep the change, or it goes back on its own.
            Rectangle {
                width: parent.width
                height: confirmRow.implicitHeight + 20
                visible: DisplaySettings.confirming
                radius: 3
                color: Scheme.dark ? "#2a3a1c" : "#fff8d6"
                border.width: 1
                border.color: Scheme.dark ? "#6b7f3c" : "#e0c66b"

                Row {
                    id: confirmRow

                    anchors.verticalCenter: parent.verticalCenter
                    x: 12
                    spacing: 12

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "Keep these display settings? Reverting in "
                            + DisplaySettings.secondsLeft + " s."
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }

                    CpButton {
                        label: "Keep changes"
                        onClicked: DisplaySettings.keep()
                    }

                    CpButton {
                        label: "Revert"
                        onClicked: DisplaySettings.revert()
                    }
                }
            }

            Note {
                width: parent.width
                text: "Your choices replace the session's layout (SHOJIWM_WIN7_OUTPUTS) "
                    + "for the screens you change; the screens are then placed side by "
                    + "side, left to right."
            }
        }
    }

    Component {
        id: devicesPage

        Column {
            id: devices

            /// The USB devices (hubs left out), the printers and the default
            /// one, from lsusb and lpstat.
            property var usb: []
            property var printers: []
            property string defaultPrinter: ""

            /// An icon of the theme that fits a device's name.
            function iconFor(name) {
                var text = name.toLowerCase();
                var kinds = [
                    ["keyboard", "input-keyboard"], ["desktop", "input-keyboard"],
                    ["mouse", "input-mouse"], ["dongle", "input-mouse"],
                    ["webcam", "camera-web"], ["camera", "camera-web"],
                    ["mv5", "audio-input-microphone"], ["microphone", "audio-input-microphone"],
                    ["audio", "audio-card"], ["headset", "audio-headset"],
                    ["monitor", "video-display"], ["display", "video-display"],
                    ["yubikey", "dialog-password"], ["security", "dialog-password"],
                    ["storage", "drive-removable-media"], ["flash", "drive-removable-media"],
                    ["bluetooth", "bluetooth"], ["led", "preferences-desktop-color"]
                ];
                for (var i = 0; i < kinds.length; i++) {
                    if (text.indexOf(kinds[i][0]) >= 0)
                        return Quickshell.iconPath(kinds[i][1], "drive-harddisk");
                }
                return Quickshell.iconPath("drive-harddisk-usb", "drive-harddisk");
            }

            spacing: 10

            Process {
                command: ["sh", "-c",
                    "echo '#usb'; lsusb 2>/dev/null | cut -d' ' -f7- | grep -v -i 'hub'"
                    + "; echo '#printers'; lpstat -p 2>/dev/null | awk '/^printer/ {print $2 \"\\t\" $4}'"
                    + "; echo '#default'; lpstat -d 2>/dev/null | sed -n 's/.*: //p'"]
                running: true
                stdout: StdioCollector {
                    onStreamFinished: {
                        var section = "";
                        var usb = [];
                        var printers = [];
                        String(text).split("\n").forEach(function (line) {
                            if (line.length === 0)
                                return;
                            if (line[0] === "#") {
                                section = line.substring(1);
                            } else if (section === "usb") {
                                usb.push(line.trim());
                            } else if (section === "printers") {
                                var parts = line.split("\t");
                                printers.push({ name: parts[0], state: parts[1] || "" });
                            } else if (section === "default") {
                                devices.defaultPrinter = line.trim();
                            }
                        });
                        devices.usb = usb;
                        devices.printers = printers;
                    }
                }
            }

            Heading { text: "Devices and Printers" }

            component Tile: Column {
                id: tile

                property string label: ""
                property string detail: ""
                property string iconSource: ""
                property bool marked: false

                width: 120
                spacing: 4

                Item {
                    width: parent.width
                    height: 52

                    IconImage {
                        anchors.centerIn: parent
                        implicitSize: 48
                        source: tile.iconSource
                    }

                    // The default one, as Windows marks it: a green tick.
                    Rectangle {
                        visible: tile.marked
                        x: parent.width / 2 + 10
                        y: 30
                        width: 18
                        height: 18
                        radius: 9
                        color: "#3fbf3a"
                        border.width: 1
                        border.color: "#ffffff"

                        Text {
                            anchors.centerIn: parent
                            text: "✔"
                            color: "#ffffff"
                            font.pixelSize: 11
                        }
                    }
                }

                Text {
                    width: parent.width
                    horizontalAlignment: Text.AlignHCenter
                    text: tile.label
                    color: Scheme.text
                    wrapMode: Text.Wrap
                    maximumLineCount: 3
                    elide: Text.ElideRight
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                }

                Text {
                    width: parent.width
                    horizontalAlignment: Text.AlignHCenter
                    visible: tile.detail.length > 0
                    text: tile.detail
                    color: Scheme.textMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall - 1
                }
            }

            SubHeading { text: "Devices (" + (devices.usb.length + Quickshell.screens.length) + ")" }

            Flow {
                width: parent.width
                spacing: 12

                Tile {
                    label: "Computer"
                    iconSource: Qt.resolvedUrl("../assets/icons/computer.png")
                }

                Repeater {
                    model: Quickshell.screens

                    delegate: Tile {
                        required property var modelData
                        // Quickshell says "Unknown" for a model it cannot read.
                        label: modelData.model && modelData.model !== "Unknown"
                            ? modelData.model : "Display"
                        detail: modelData.name + " · " + modelData.width + " × " + modelData.height
                        iconSource: Quickshell.iconPath("video-display", "computer")
                    }
                }

                Repeater {
                    model: devices.usb

                    delegate: Tile {
                        required property var modelData
                        label: modelData
                        detail: "USB"
                        iconSource: devices.iconFor(modelData)
                    }
                }
            }

            SubHeading { text: "Printers (" + devices.printers.length + ")" }

            Flow {
                width: parent.width
                spacing: 12

                Repeater {
                    model: devices.printers

                    delegate: Tile {
                        required property var modelData
                        label: modelData.name.replace(/_/g, " ")
                        detail: modelData.state
                        iconSource: Quickshell.iconPath("printer", "document-print")
                        marked: modelData.name === devices.defaultPrinter
                    }
                }
            }

            Note {
                width: parent.width
                visible: devices.printers.length === 0
                text: "No printer is set up (CUPS)."
            }
        }
    }

    Component {
        id: soundPage

        Column {
            id: sound

            spacing: 10

            // The devices: audio nodes that are not a program's stream.
            readonly property var devices: Pipewire.nodes ? Pipewire.nodes.values.filter(function (node) {
                return node && node.audio && !node.isStream;
            }) : []
            readonly property var playback: sound.devices.filter(function (node) { return node.isSink; })
            readonly property var recording: sound.devices.filter(function (node) { return !node.isSink; })

            Heading { text: "Sound" }

            component DeviceRow: Rectangle {
                id: device

                required property var modelData
                property bool isDefault: false
                property bool playbackDevice: true

                width: parent ? parent.width : 400
                height: 46
                radius: Scheme.itemRadius
                color: device.isDefault ? Scheme.selection : "transparent"
                border.width: device.isDefault ? 1 : 0
                border.color: Scheme.selectionBorder

                Image {
                    x: 10
                    anchors.verticalCenter: parent.verticalCenter
                    width: 28
                    height: 28
                    sourceSize: Qt.size(56, 56)
                    source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-") + "speaker-3.svg")
                    visible: device.playbackDevice
                }

                Text {
                    x: 10
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !device.playbackDevice
                    text: "🎙"
                    font.pixelSize: 22
                }

                Column {
                    x: 50
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - 50 - 130

                    Text {
                        width: parent.width
                        text: device.modelData.nickname || device.modelData.description || device.modelData.name
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        text: device.isDefault ? "Default Device" : device.modelData.description
                        color: device.isDefault ? "#3fbf3a" : Scheme.textMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeSmall
                        elide: Text.ElideRight
                    }
                }

                CpButton {
                    anchors.right: parent.right
                    anchors.rightMargin: 10
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !device.isDefault
                    label: "Set Default"
                    onClicked: {
                        if (device.playbackDevice)
                            Pipewire.preferredDefaultAudioSink = device.modelData;
                        else
                            Pipewire.preferredDefaultAudioSource = device.modelData;
                    }
                }
            }

            SubHeading { text: "Playback" }

            Repeater {
                model: sound.playback

                delegate: DeviceRow {
                    width: sound.width
                    isDefault: modelData === Pipewire.defaultAudioSink
                    playbackDevice: true
                }
            }

            SubHeading { text: "Recording" }

            Repeater {
                model: sound.recording

                delegate: DeviceRow {
                    width: sound.width
                    isDefault: modelData === Pipewire.defaultAudioSource
                    playbackDevice: false
                }
            }

            Note {
                width: parent.width
                visible: sound.devices.length === 0
                text: "PipeWire lists no sound device."
            }

            CpLink {
                topPadding: 6
                text: "Open the volume mixer"
                onClicked: Audio.openMixer()
            }
        }
    }

    Component {
        id: defaultsPage

        Column {
            spacing: 12

            Component.onCompleted: DefaultApps.refresh()

            Heading { text: "Choose the programs that open your files and links" }

            Note {
                width: parent.width
                text: "The choice applies to every program that opens files through "
                    + "the desktop's standard (xdg-open), and is kept in ~/.config/mimeapps.list."
            }

            Repeater {
                model: DefaultApps.kinds

                delegate: Row {
                    id: kindRow

                    required property var modelData
                    required property int index

                    readonly property var info: DefaultApps.state[kindRow.modelData.types[0]]
                        || { current: "", candidates: [] }
                    /// The programs to choose from, the current one first.
                    readonly property var choices: {
                        var list = kindRow.info.candidates.slice();
                        if (kindRow.info.current.length > 0 && list.indexOf(kindRow.info.current) < 0)
                            list.unshift(kindRow.info.current);
                        return list;
                    }

                    function nameOf(id) {
                        var entry = Apps.entryById(id);
                        return entry ? String(entry.name) : id.replace(/\.desktop$/, "");
                    }

                    spacing: 12
                    // The open lists draw over the rows below.
                    z: 100 - kindRow.index

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 120
                        text: kindRow.modelData.label
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }

                    IconImage {
                        anchors.verticalCenter: parent.verticalCenter
                        implicitSize: 24
                        source: kindRow.info.current.length > 0
                            ? Apps.entryIcon(Apps.entryById(kindRow.info.current)) : ""
                    }

                    CpCombo {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 280
                        model: kindRow.choices.length > 0
                            ? kindRow.choices.map(function (id) { return kindRow.nameOf(id); })
                            : ["No program says it can"]
                        currentIndex: Math.max(0, kindRow.choices.indexOf(kindRow.info.current))
                        onActivated: function (index) {
                            if (kindRow.choices[index])
                                DefaultApps.setDefault(kindRow.modelData, kindRow.choices[index]);
                        }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: kindRow.info.current.length === 0
                        text: "Not set"
                        color: Scheme.textMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }
        }
    }

    Component {
        id: startupPage

        Column {
            id: startup

            /// The list of programs to add one from is open.
            property bool adding: false
            property string filter: ""

            spacing: 10

            Component.onCompleted: Startup.refresh()

            Heading { text: "Choose the programs that start when you log on" }

            Note {
                width: parent.width
                text: "These are the XDG autostart entries of the system and your own, "
                    + "in " + Startup.userDir + ". A change applies the next time you log on."
            }

            Item { width: 1; height: 4 }

            Repeater {
                model: Startup.entries

                delegate: Row {
                    id: startupRow

                    required property var modelData

                    spacing: 12
                    opacity: startupRow.modelData.here ? 1 : 0.55

                    CpCheck {
                        anchors.verticalCenter: parent.verticalCenter
                        label: ""
                        checked: startupRow.modelData.enabled
                        onToggled: function (on) { Startup.setEnabled(startupRow.modelData, on); }
                    }

                    IconImage {
                        anchors.verticalCenter: parent.verticalCenter
                        implicitSize: 24
                        source: Quickshell.iconPath(startupRow.modelData.icon, "application-x-executable")
                    }

                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 330

                        Text {
                            width: parent.width
                            text: startupRow.modelData.name
                                + (startupRow.modelData.here ? "" : " (not in this desktop)")
                            color: Scheme.text
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSize
                            elide: Text.ElideRight
                        }

                        Text {
                            width: parent.width
                            text: startupRow.modelData.exec
                            color: Scheme.textMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            elide: Text.ElideMiddle
                        }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 70
                        text: startupRow.modelData.systemPath.length > 0 ? "System" : "Yours"
                        color: Scheme.textMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeSmall
                    }

                    // An entry you added goes; a system entry you changed
                    // goes back to the system's.
                    CpLink {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: startupRow.modelData.userPath.length > 0
                        text: startupRow.modelData.systemPath.length > 0 ? "Restore" : "Remove"
                        onClicked: Startup.removeUserFile(startupRow.modelData)
                    }
                }
            }

            Note {
                width: parent.width
                visible: Startup.entries.length === 0
                text: "No program starts when you log on."
            }

            Row {
                spacing: 10
                topPadding: 8

                CpButton {
                    label: startup.adding ? "Done" : "Add a program…"
                    onClicked: {
                        startup.adding = !startup.adding;
                        startup.filter = "";
                    }
                }
            }

            // The installed programs, to pick one that starts at log-on.
            Column {
                width: parent.width
                visible: startup.adding
                spacing: 6

                Rectangle {
                    width: 260
                    height: 24
                    radius: Scheme.fieldRadius
                    color: Scheme.field
                    border.width: 1
                    border.color: filterField.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

                    TextInput {
                        id: filterField
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        anchors.rightMargin: 6
                        verticalAlignment: TextInput.AlignVCenter
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                        selectByMouse: true
                        clip: true
                        onTextChanged: startup.filter = text.toLowerCase()

                        Text {
                            anchors.fill: parent
                            verticalAlignment: Text.AlignVCenter
                            visible: filterField.text.length === 0
                            text: "Find a program"
                            color: Scheme.placeholder
                            font: filterField.font
                        }
                    }
                }

                Repeater {
                    model: startup.adding ? Apps.applications.filter(function (entry) {
                        var id = Apps.entryFileId(entry);
                        return Startup.entries.every(function (item) { return item.id !== id; })
                            && (startup.filter.length === 0
                                || String(entry.name).toLowerCase().indexOf(startup.filter) >= 0);
                    }).slice(0, 12) : []

                    delegate: Rectangle {
                        id: candidate

                        required property var modelData

                        width: 360
                        height: 28
                        radius: Scheme.itemRadius
                        color: candidateArea.containsMouse ? Scheme.hover : "transparent"
                        border.width: candidateArea.containsMouse ? 1 : 0
                        border.color: Scheme.selectionBorder

                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: 6
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 8

                            IconImage {
                                anchors.verticalCenter: parent.verticalCenter
                                implicitSize: 20
                                source: Quickshell.iconPath(candidate.modelData.icon, "application-x-executable")
                            }

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: candidate.modelData.name
                                color: Scheme.text
                                font.family: Theme.fontFamily
                                font.pixelSize: Theme.fontSize
                            }
                        }

                        MouseArea {
                            id: candidateArea
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: Startup.add(Apps.entryFileId(candidate.modelData))
                        }
                    }
                }
            }
        }
    }

    Component {
        id: homePage

        Column {
            spacing: 18

            Heading { text: "Adjust your computer's settings" }

            Grid {
                columns: 2
                columnSpacing: 40
                rowSpacing: 26

                Category {
                    icon: "pictures"
                    name: "Appearance and Personalization"
                    target: "appearance"
                    links: [
                        { text: "Choose light or dark", page: "appearance" },
                        { text: "Change desktop background", page: "background" },
                        { text: "Taskbar and Start menu", page: "taskbar" },
                        { text: "Notification area icons", page: "notifications" }
                    ]
                }

                Category {
                    icon: "control-panel"
                    name: "User Accounts"
                    target: "account"
                    links: [
                        { text: "Change your account picture", page: "account" },
                        { text: "Lock screen and its picture", page: "lockscreen" }
                    ]
                }

                Category {
                    icon: "calendar"
                    name: "Clock and Region"
                    target: "clock"
                    links: [
                        { text: "Change the clock format", page: "clock" }
                    ]
                }

                Category {
                    icon: "run"
                    name: "Ease of Access"
                    target: "ease"
                    links: [
                        { text: "Turn off all unnecessary animations", page: "ease" }
                    ]
                }

                Category {
                    icon: "computer"
                    name: "System"
                    target: "about"
                    links: [
                        { text: "Choose the programs that open your files and links", page: "defaults" },
                        { text: "Choose the programs that start when you log on", page: "startup" },
                        { text: "About this desktop", page: "about" }
                    ]
                }
            }
        }
    }

    Component {
        id: appearancePage

        Column {
            spacing: 12

            Heading { text: "Choose light or dark" }

            Note {
                width: parent.width
                text: "This is the system's own setting: programs that follow it "
                    + "(GTK, libadwaita, Qt, Firefox, Chromium) change "
                    + "with it, and so do the menus of this desktop. "
                    + "The taskbar and the window frames stay dark glass, as "
                    + "on Windows 7."
            }

            Row {
                spacing: 18

                Repeater {
                    model: [
                        { dark: false, label: "Light" },
                        { dark: true, label: "Dark" }
                    ]

                    delegate: Item {
                        id: choice

                        required property var modelData
                        readonly property bool isChosen: Settings.dark === modelData.dark

                        width: 200
                        height: 150

                        Rectangle {
                            anchors.fill: parent
                            radius: 4
                            color: choice.isChosen ? Scheme.selection
                                : choiceArea.containsMouse ? Scheme.hover : "transparent"
                            border.width: choice.isChosen || choiceArea.containsMouse ? 1 : 0
                            border.color: Scheme.selectionBorder
                        }

                        // A small window in that scheme.
                        Rectangle {
                            x: 16
                            y: 12
                            width: parent.width - 32
                            height: 96
                            radius: 4
                            border.width: 1
                            border.color: "#5d7489"
                            color: choice.modelData.dark ? "#1e262e" : "#ffffff"

                            Rectangle {
                                width: parent.width
                                height: 18
                                radius: 4
                                gradient: Gradient {
                                    GradientStop { position: 0.0; color: "#a6c8e6" }
                                    GradientStop { position: 1.0; color: "#6f9cc4" }
                                }
                            }

                            Column {
                                x: 10
                                y: 28
                                spacing: 6

                                Repeater {
                                    model: 4
                                    delegate: Rectangle {
                                        required property int index
                                        width: 120 - index * 18
                                        height: 6
                                        radius: 3
                                        color: choice.modelData.dark ? "#5d7489" : "#c3d3e0"
                                    }
                                }
                            }
                        }

                        CpRadio {
                            anchors.horizontalCenter: parent.horizontalCenter
                            anchors.bottom: parent.bottom
                            anchors.bottomMargin: 10
                            label: choice.modelData.label
                            checked: choice.isChosen
                            onClicked: Settings.setDark(choice.modelData.dark)
                        }

                        MouseArea {
                            id: choiceArea
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: Settings.setDark(choice.modelData.dark)
                        }
                    }
                }
            }

            Note {
                width: parent.width
                visible: !Settings.systemSchemeAvailable
                color: "#c42b1c"
                text: "dconf could not be reached, so programs will not see this "
                    + "choice; only this desktop will, until it restarts. "
                    + "On NixOS: programs.dconf.enable = true (the shojiwm "
                    + "module of nixconfig sets it)."
            }

            SubHeading { text: "Glass" }

            Note {
                width: parent.width
                text: "How much the caption bars, the taskbar and the menus blur "
                    + "what is behind them. More blur costs more of the graphics card."
            }

            Flow {
                width: parent.width
                spacing: 16

                Repeater {
                    model: [
                        { level: 0, label: "Clear" },
                        { level: 1, label: "Light" },
                        { level: 2, label: "Medium" },
                        { level: 3, label: "Strong" },
                        { level: 4, label: "Frosted" }
                    ]

                    delegate: CpRadio {
                        required property var modelData
                        label: modelData.label
                        checked: Settings.glassBlur === modelData.level
                        onClicked: Settings.glassBlur = modelData.level
                    }
                }
            }

            Note {
                width: parent.width
                text: "The colour of the taskbar's and the Start menu's glass:"
            }

            Flow {
                width: parent.width
                spacing: 8

                Repeater {
                    model: [
                        { color: "", name: "Plain glass" },
                        { color: "#3b8fd6", name: "Sky" },
                        { color: "#1f5aa8", name: "Twilight" },
                        { color: "#1aa2a8", name: "Sea" },
                        { color: "#4fa83d", name: "Leaf" },
                        { color: "#9cc93a", name: "Lime" },
                        { color: "#e8c22f", name: "Sun" },
                        { color: "#e8892f", name: "Pumpkin" },
                        { color: "#e0607e", name: "Blush" },
                        { color: "#a64fb3", name: "Violet" },
                        { color: "#b49bd9", name: "Lavender" },
                        { color: "#a69b85", name: "Taupe" },
                        { color: "#6b4a2f", name: "Chocolate" },
                        { color: "#6b7b8c", name: "Slate" },
                        { color: "#e6eef2", name: "Frost" }
                    ]

                    delegate: Rectangle {
                        id: swatch

                        required property var modelData
                        readonly property bool chosen: Settings.glassColor === swatch.modelData.color

                        width: 40
                        height: 40
                        radius: 4
                        border.width: swatch.chosen ? 2 : 1
                        border.color: swatch.chosen ? Scheme.fieldFocus
                            : swatchArea.containsMouse ? Scheme.selectionBorder : Scheme.fieldBorder
                        // The glass of the taskbar under the colour, as the
                        // swatches of Windows 7 show it.
                        gradient: Gradient {
                            GradientStop { position: 0.0; color: "#7fa3c4" }
                            GradientStop { position: 1.0; color: "#1d3550" }
                        }

                        Rectangle {
                            anchors.fill: parent
                            anchors.margins: 3
                            radius: 2
                            color: swatch.modelData.color.length > 0 ? swatch.modelData.color : "transparent"
                            opacity: 0.75
                        }

                        MouseArea {
                            id: swatchArea
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: Settings.glassColor = swatch.modelData.color
                        }
                    }
                }
            }

            Text {
                text: "Colour: " + ({ "": "Plain glass", "#3b8fd6": "Sky", "#1f5aa8": "Twilight",
                    "#1aa2a8": "Sea", "#4fa83d": "Leaf", "#9cc93a": "Lime", "#e8c22f": "Sun",
                    "#e8892f": "Pumpkin", "#e0607e": "Blush", "#a64fb3": "Violet",
                    "#b49bd9": "Lavender", "#a69b85": "Taupe", "#6b4a2f": "Chocolate",
                    "#6b7b8c": "Slate", "#e6eef2": "Frost" })[Settings.glassColor] || Settings.glassColor
                color: Scheme.text
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
            }

            Row {
                spacing: 12
                visible: Settings.glassColor.length > 0

                Note {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Colour intensity:"
                }

                // From faint to strong, as Windows 7's intensity slider.
                Item {
                    id: strength

                    anchors.verticalCenter: parent.verticalCenter
                    width: 220
                    height: 24

                    function setFrom(x) {
                        Settings.glassColorStrength = Math.round(
                            (0.1 + Math.max(0, Math.min(1, x / width)) * 0.8) * 100) / 100;
                    }

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width
                        height: 4
                        radius: 2
                        color: Scheme.field
                        border.width: 1
                        border.color: Scheme.fieldBorder
                    }

                    Rectangle {
                        x: (Settings.glassColorStrength - 0.1) / 0.8 * strength.width - width / 2
                        anchors.verticalCenter: parent.verticalCenter
                        width: 10
                        height: 20
                        radius: 2
                        border.width: 1
                        border.color: strengthArea.containsMouse || strengthArea.pressed
                            ? Scheme.fieldFocus : Scheme.fieldBorder
                        gradient: Gradient {
                            GradientStop { position: 0.0; color: strengthArea.containsMouse ? Scheme.buttonHotTop : Scheme.buttonTop }
                            GradientStop { position: 1.0; color: strengthArea.containsMouse ? Scheme.buttonHotBottom : Scheme.buttonBottom }
                        }
                    }

                    MouseArea {
                        id: strengthArea
                        anchors.fill: parent
                        anchors.leftMargin: -6
                        anchors.rightMargin: -6
                        hoverEnabled: true
                        preventStealing: true
                        onPressed: function (event) { strength.setFrom(event.x - 6); }
                        onPositionChanged: function (event) {
                            if (pressed)
                                strength.setFrom(event.x - 6);
                        }
                    }
                }
            }
        }
    }

    Component {
        id: backgroundPage

        Column {
            spacing: 12

            Heading { text: "Choose your desktop background" }
            Note {
                width: parent.width
                text: "Click a picture to make it your desktop background."
            }

            PictureChooser {
                width: parent.width
                selected: Settings.wallpaper
                onChosen: function (path) { Settings.wallpaper = path; }
            }

            Row {
                spacing: 12
                // The open list draws over the slide show below.
                z: 5

                Note {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Picture position:"
                }

                CpCombo {
                    readonly property var positions: ["fill", "fit", "stretch", "tile", "center"]
                    width: 160
                    model: ["Fill", "Fit", "Stretch", "Tile", "Center"]
                    currentIndex: Math.max(0, positions.indexOf(Settings.wallpaperPosition))
                    onActivated: function (index) { Settings.wallpaperPosition = positions[index]; }
                }
            }

            SubHeading { text: "Slide show" }

            CpCheck {
                label: "Change the picture on its own, from a folder"
                checked: Settings.slideshowFolder.length > 0
                onToggled: function (on) {
                    Settings.slideshowFolder = on ? Apps.userDir("PICTURES", "Pictures") : "";
                }
            }

            Grid {
                columns: 2
                columnSpacing: 12
                rowSpacing: 8
                leftPadding: 24
                visible: Settings.slideshowFolder.length > 0
                z: 5

                Note { text: "Folder:"; height: 24; verticalAlignment: Text.AlignVCenter }
                Rectangle {
                    width: 320
                    height: 24
                    radius: Scheme.fieldRadius
                    color: Scheme.field
                    border.width: 1
                    border.color: folderField.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

                    TextInput {
                        id: folderField
                        anchors.fill: parent
                        anchors.leftMargin: 6
                        anchors.rightMargin: 6
                        verticalAlignment: TextInput.AlignVCenter
                        text: Settings.slideshowFolder
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                        selectByMouse: true
                        clip: true
                        onEditingFinished: {
                            if (text.length > 0 && text !== Settings.slideshowFolder)
                                Settings.slideshowFolder = text;
                        }
                    }
                }

                Note { text: "Change every:"; height: 24; verticalAlignment: Text.AlignVCenter }
                CpCombo {
                    readonly property var minutes: [1, 10, 30, 60, 360, 1440]
                    width: 160
                    z: open ? 10 : 1
                    model: ["1 minute", "10 minutes", "30 minutes", "1 hour", "6 hours", "1 day"]
                    currentIndex: Math.max(0, minutes.indexOf(Settings.slideshowMinutes))
                    onActivated: function (index) { Settings.slideshowMinutes = minutes[index]; }
                }

                Item { width: 1; height: 1 }
                CpCheck {
                    label: "Shuffle"
                    checked: Settings.slideshowShuffle
                    onToggled: function (on) { Settings.slideshowShuffle = on; }
                }

                Item { width: 1; height: 1 }
                CpButton {
                    label: "Next picture"
                    onClicked: Slideshow.next()
                }
            }
        }
    }

    Component {
        id: lockPage

        Column {
            spacing: 12

            Heading { text: "Lock screen" }

            SubHeading { text: "Picture" }

            CpCheck {
                label: "Use the desktop background"
                checked: Settings.lockBackground.length === 0
                onToggled: function (on) {
                    Settings.lockBackground = on ? "" : (Settings.wallpaper || "builtin");
                }
            }

            PictureChooser {
                width: parent.width
                visible: Settings.lockBackground.length > 0
                selected: Settings.lockBackground === "builtin" ? "" : Settings.lockBackground
                builtInLabel: "Windows 7 style (built in)"
                onChosen: function (path) {
                    // "" means "the desktop's": keep a real choice instead.
                    Settings.lockBackground = path.length > 0 ? path : "builtin";
                }
            }

            SubHeading { text: "Screen saver" }

            Row {
                spacing: 12
                z: 6

                CpCombo {
                    readonly property var savers: ["none", "mystify"]
                    width: 160
                    model: ["(None)", "Mystify"]
                    currentIndex: Math.max(0, savers.indexOf(Settings.screensaver))
                    onActivated: function (index) { Settings.screensaver = savers[index]; }
                }

                Note {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Wait:"
                }

                CpCombo {
                    readonly property var minutes: [1, 2, 5, 10, 15, 30]
                    width: 120
                    z: 1
                    model: ["1 minute", "2 minutes", "5 minutes", "10 minutes", "15 minutes", "30 minutes"]
                    currentIndex: Math.max(0, minutes.indexOf(Settings.screensaverMinutes))
                    onActivated: function (index) { Settings.screensaverMinutes = minutes[index]; }
                }

                CpButton {
                    label: "Preview"
                    visible: Settings.screensaver !== "none"
                    onClicked: Settings.screensaverPreview = true
                }
            }

            Note {
                width: parent.width
                text: "It runs while the computer is left alone, before the lock; the mouse or a key ends it."
            }

            SubHeading { text: "Lock the computer" }

            Note { text: "After the mouse and the keyboard have been left alone for:" }

            Flow {
                width: parent.width
                spacing: 16

                Repeater {
                    model: [
                        { minutes: 1, label: "1 minute" },
                        { minutes: 5, label: "5 minutes" },
                        { minutes: 10, label: "10 minutes" },
                        { minutes: 15, label: "15 minutes" },
                        { minutes: 30, label: "30 minutes" },
                        { minutes: 60, label: "1 hour" },
                        { minutes: 0, label: "Never" }
                    ]

                    delegate: CpRadio {
                        required property var modelData
                        label: modelData.label
                        checked: Settings.lockAfterMinutes === modelData.minutes
                        onClicked: Settings.lockAfterMinutes = modelData.minutes
                    }
                }
            }

            CpCheck {
                label: "Lock when the computer goes to sleep"
                checked: Settings.lockOnSuspend
                onToggled: function (on) { Settings.lockOnSuspend = on; }
            }

            Row {
                spacing: 10
                topPadding: 8

                CpButton {
                    label: "Lock now"
                    onClicked: LockState.lock()
                }

                Note {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Also Super+L, and loginctl lock-session."
                }
            }
        }
    }

    Component {
        id: accountPage

        Column {
            spacing: 12

            Heading { text: "Choose your account picture" }
            Note {
                width: parent.width
                text: "Shown in the Start menu and on the lock screen. With none "
                    + "chosen here, the system's is used, as log-in screens read it: "
                    + "AccountsService's icon, ~/.face, then ~/.face.icon."
            }

            Row {
                spacing: 16

                Rectangle {
                    width: 96
                    height: 96
                    radius: 6
                    color: "#ffffff"
                    border.width: 1
                    border.color: Scheme.fieldBorder
                    clip: true

                    Image {
                        anchors.fill: parent
                        anchors.margins: 4
                        source: Settings.userPictureSource
                        sourceSize.width: 192
                        sourceSize.height: 192
                        fillMode: Image.PreserveAspectCrop
                    }
                }

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 6

                    Text {
                        text: Apps.userName
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize + 4
                    }

                    CpLink {
                        visible: Settings.userPicture.length > 0
                        text: "Go back to the default picture"
                        onClicked: Settings.userPicture = ""
                    }
                }
            }

            PictureChooser {
                width: parent.width
                square: true
                selected: Settings.userPicture
                builtIn: Qt.resolvedUrl("../assets/user.svg")
                builtInLabel: "The default picture"
                onChosen: function (path) { Settings.userPicture = path; }
            }
        }
    }

    Component {
        id: taskbarPage

        Column {
            spacing: 10

            Heading { text: "Taskbar and Start menu" }

            SubHeading { text: "Taskbar" }

            CpCheck {
                label: "Combine the windows of a program into one button"
                checked: Settings.combineWindows
                onToggled: function (on) { Settings.combineWindows = on; }
            }

            CpCheck {
                label: "Compact: show the icons alone, without labels"
                checked: Settings.taskbarCompact
                onToggled: function (on) { Settings.taskbarCompact = on; }
            }

            CpCheck {
                label: "Auto-hide the taskbar"
                checked: Settings.taskbarAutoHide
                onToggled: function (on) { Settings.taskbarAutoHide = on; }
            }

            CpCheck {
                label: "Use Aero Peek to preview the desktop"
                checked: Settings.peek
                onToggled: function (on) { Settings.peek = on; }
            }

            SubHeading { text: "Desktop icons" }

            Note { text: "Show these on the desktop, beside the Recycle Bin:" }

            Repeater {
                model: [
                    { kind: "computer", label: "Computer" },
                    { kind: "userfiles", label: "User's Files" },
                    { kind: "controlpanel", label: "Control Panel" }
                ]

                delegate: CpCheck {
                    required property var modelData
                    label: modelData.label
                    checked: Settings.desktopSystemIcons.indexOf(modelData.kind) >= 0
                    onToggled: function (on) {
                        var kinds = Settings.desktopSystemIcons.filter(function (kind) {
                            return kind !== modelData.kind;
                        });
                        if (on)
                            kinds.push(modelData.kind);
                        Settings.desktopSystemIcons = kinds;
                    }
                }
            }

            SubHeading { text: "Start menu" }

            CpCheck {
                label: "Use large icons"
                checked: Settings.startMenuLargeIcons
                onToggled: function (on) { Settings.startMenuLargeIcons = on; }
            }

            CpCheck {
                label: "Show the programs used most"
                checked: Settings.showFrequent
                onToggled: function (on) { Settings.showFrequent = on; }
            }

            Note { text: "The power button:" }

            Flow {
                width: parent.width
                spacing: 16

                Repeater {
                    model: Apps.powerActions.filter(function (action) {
                        return action.id !== "separator";
                    })

                    delegate: CpRadio {
                        required property var modelData
                        label: modelData.label
                        checked: Apps.powerButtonAction === modelData.id
                        onClicked: Settings.powerButtonAction = modelData.id
                    }
                }
            }

            Note { text: "What the Shut Down dialog (Super+Shift+E) selects:" }

            Flow {
                width: parent.width
                spacing: 16

                Repeater {
                    model: Apps.powerActions.filter(function (action) {
                        return action.id !== "separator";
                    })

                    delegate: CpRadio {
                        required property var modelData
                        label: modelData.label
                        checked: Apps.shutdownDialogAction === modelData.id
                        onClicked: Settings.shutdownDialogAction = modelData.id
                    }
                }
            }
        }
    }

    Component {
        id: notificationsPage

        Column {
            spacing: 10

            Heading { text: "Select which icons appear on the taskbar" }

            Note {
                width: parent.width
                text: "An icon left off the taskbar waits in the popover the "
                    + "arrow beside the clock opens. A program that starts "
                    + "later puts its icon there until you choose otherwise."
            }

            CpCheck {
                label: "Always show all icons on the taskbar"
                checked: Settings.trayShowAll
                onToggled: function (on) { Settings.trayShowAll = on; }
            }

            Item { width: 1; height: 4 }

            Repeater {
                model: SystemTray.items ? SystemTray.items.values : []

                delegate: Row {
                    id: trayRow

                    required property var modelData

                    spacing: 12
                    opacity: Settings.trayShowAll ? 0.5 : 1

                    IconImage {
                        anchors.verticalCenter: parent.verticalCenter
                        implicitSize: 20
                        source: trayRow.modelData && trayRow.modelData.icon
                            ? trayRow.modelData.icon : ""
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 260
                        text: trayRow.modelData.tooltipTitle
                            || trayRow.modelData.title || trayRow.modelData.id
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                        elide: Text.ElideRight
                    }

                    CpCheck {
                        anchors.verticalCenter: parent.verticalCenter
                        label: "Show on the taskbar"
                        checked: Settings.trayVisible(trayRow.modelData.id)
                        onToggled: function (on) {
                            Settings.setTrayVisible(trayRow.modelData.id, on);
                        }
                    }
                }
            }

            Note {
                width: parent.width
                visible: !SystemTray.items || SystemTray.items.values.length === 0
                text: "No program has an icon in the notification area now."
            }
        }
    }

    Component {
        id: clockPage

        Column {
            spacing: 10

            Heading { text: "Date and time" }

            SubHeading { text: "Clock format" }

            Repeater {
                model: [
                    { value: "locale", label: "As the region sets it (" + Qt.locale().name + ")" },
                    { value: "24h", label: "24-hour (17:45)" },
                    { value: "12h", label: "12-hour (5:45 PM)" }
                ]

                delegate: CpRadio {
                    required property var modelData
                    label: modelData.label
                    checked: Settings.clockFormat === modelData.value
                    onClicked: Settings.clockFormat = modelData.value
                }
            }

            CpCheck {
                label: "Show the date under the time"
                checked: Settings.showDate
                onToggled: function (on) { Settings.showDate = on; }
            }

            Note {
                width: parent.width
                text: "The language and the date format follow the system locale "
                    + "(LANG, i18n.defaultLocale on NixOS)."
            }

            SubHeading { text: "Additional clocks" }

            Note {
                width: parent.width
                text: "Show the time in other time zones under the calendar and "
                    + "when you rest the pointer on the clock."
            }

            Repeater {
                model: 2

                delegate: Column {
                    id: extra

                    required property int index

                    readonly property var clocks: Settings.extraClocks || []
                    readonly property bool shown: extra.index < extra.clocks.length
                    readonly property var clock: extra.shown ? extra.clocks[extra.index] : null
                    /// Zones that hold what is typed, for the field to offer.
                    readonly property var matches: {
                        var typed = zoneField.text.toLowerCase().replace(/ /g, "_");
                        if (!zoneField.activeFocus || typed.length < 2)
                            return [];
                        return WorldClocks.zones.filter(function (zone) {
                            return zone.toLowerCase().indexOf(typed) >= 0 && zone !== zoneField.text;
                        }).slice(0, 6);
                    }

                    function store(zone, label) {
                        var list = extra.clocks.slice();
                        list[extra.index] = { zone: zone, label: label };
                        Settings.extraClocks = list;
                    }

                    spacing: 6
                    // A second clock comes after a first.
                    visible: extra.index <= extra.clocks.length

                    CpCheck {
                        label: "Show this clock"
                        checked: extra.shown
                        onToggled: function (on) {
                            var list = extra.clocks.slice();
                            if (on)
                                list.push({ zone: "America/New_York", label: "New York" });
                            else
                                list.splice(extra.index, 1);
                            Settings.extraClocks = list;
                        }
                    }

                    Grid {
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 6
                        leftPadding: 24
                        visible: extra.shown

                        Note { text: "Display name:"; height: 24; verticalAlignment: Text.AlignVCenter }
                        Rectangle {
                            width: 260
                            height: 24
                            radius: Scheme.fieldRadius
                            color: Scheme.field
                            border.width: 1
                            border.color: labelField.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

                            TextInput {
                                id: labelField
                                // A click picks the whole value, so typing replaces it.
                                onActiveFocusChanged: if (activeFocus) selectAll()
                                anchors.fill: parent
                                anchors.leftMargin: 6
                                anchors.rightMargin: 6
                                verticalAlignment: TextInput.AlignVCenter
                                text: extra.clock ? extra.clock.label : ""
                                color: Scheme.text
                                font.family: Theme.fontFamily
                                font.pixelSize: Theme.fontSize
                                selectByMouse: true
                                clip: true
                                onEditingFinished: {
                                    if (extra.clock && text !== extra.clock.label)
                                        extra.store(extra.clock.zone, text);
                                }
                            }
                        }

                        Note { text: "Time zone:"; height: 24; verticalAlignment: Text.AlignVCenter }
                        Column {
                            spacing: 2

                            Rectangle {
                                width: 260
                                height: 24
                                radius: Scheme.fieldRadius
                                color: Scheme.field
                                border.width: 1
                                border.color: zoneField.activeFocus ? Scheme.fieldFocus
                                    : WorldClocks.zones.indexOf(zoneField.text) >= 0
                                        ? Scheme.fieldBorder : "#d9302a"

                                TextInput {
                                    id: zoneField
                                // A click picks the whole value, so typing replaces it.
                                onActiveFocusChanged: if (activeFocus) selectAll()
                                    anchors.fill: parent
                                    anchors.leftMargin: 6
                                    anchors.rightMargin: 6
                                    verticalAlignment: TextInput.AlignVCenter
                                    text: extra.clock ? extra.clock.zone : ""
                                    color: Scheme.text
                                    font.family: Theme.fontFamily
                                    font.pixelSize: Theme.fontSize
                                    selectByMouse: true
                                    clip: true
                                    // A zone the system knows is kept.
                                    onEditingFinished: {
                                        if (extra.clock && WorldClocks.zones.indexOf(text) >= 0
                                            && text !== extra.clock.zone)
                                            extra.store(text, extra.clock.label);
                                    }
                                }
                            }

                            Repeater {
                                model: extra.matches

                                delegate: CpLink {
                                    required property var modelData
                                    text: modelData.replace(/_/g, " ")
                                    onClicked: {
                                        var name = modelData.substring(modelData.lastIndexOf("/") + 1)
                                            .replace(/_/g, " ");
                                        extra.store(modelData, name);
                                        zoneField.focus = false;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SubHeading { text: "Time zone and synchronization" }

            // What systemd reports. The system's configuration sets these
            // (time.timeZone and the time servers on NixOS), so they are shown,
            // not changed, here.
            Process {
                id: timedate
                command: ["timedatectl", "show"]
                running: true
                stdout: StdioCollector {
                    onStreamFinished: {
                        var values = {};
                        String(text).split("\n").forEach(function (line) {
                            var at = line.indexOf("=");
                            if (at > 0)
                                values[line.substring(0, at)] = line.substring(at + 1);
                        });
                        timeInfo.values = values;
                    }
                }
            }

            Item {
                id: timeInfo
                property var values: ({})
                width: 1
                height: 1
            }

            Repeater {
                model: [
                    { label: "Time zone", value: timeInfo.values.Timezone || "unknown" },
                    { label: "Synchronized with an Internet time server",
                      value: timeInfo.values.NTPSynchronized === "yes" ? "Yes"
                          : timeInfo.values.NTPSynchronized === "no" ? "No" : "unknown" },
                    { label: "Network time", value: timeInfo.values.NTP === "yes" ? "On"
                          : timeInfo.values.NTP === "no" ? "Off" : "unknown" }
                ]

                delegate: Text {
                    required property var modelData
                    text: modelData.label + ":  " + modelData.value
                    color: Scheme.text
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }
            }

            Note {
                width: parent.width
                text: "Set by the system's configuration (time.timeZone on NixOS), "
                    + "so it is shown here and changed there."
            }
        }
    }

    Component {
        id: aboutPage

        Column {
            spacing: 10

            Heading { text: "View basic information about your computer" }

            // What Windows 7's System page shows, read from the system: the
            // edition, the processor, the memory, the system type, the
            // computer's name, and how long it has been on.
            Process {
                id: systemInfo
                command: ["sh", "-c",
                    ". /etc/os-release 2>/dev/null; echo \"$PRETTY_NAME\";"
                    + " grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | sed 's/^ *//';"
                    + " nproc; awk '/MemTotal/ {print $2}' /proc/meminfo; uname -m;"
                    + " cat /proc/sys/kernel/hostname; uname -r; cut -d' ' -f1 /proc/uptime"]
                running: true
                stdout: StdioCollector {
                    onStreamFinished: systemFacts.lines = String(text).split("\n")
                }
            }

            Item {
                id: systemFacts
                property var lines: []
                readonly property string uptime: {
                    var seconds = Number(lines[7]) || 0;
                    var days = Math.floor(seconds / 86400);
                    var hours = Math.floor(seconds % 86400 / 3600);
                    var minutes = Math.floor(seconds % 3600 / 60);
                    return (days > 0 ? days + " d " : "") + hours + " h " + minutes + " min";
                }
                width: 1
                height: 1
            }

            // One line of facts: the label in a column of one width, so the
            // values line up through every section.
            component InfoRow: Row {
                property string label: ""
                property string value: ""

                spacing: 12

                Text {
                    width: 190
                    text: parent.label + ":"
                    color: Scheme.textMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }

                Text {
                    text: parent.value
                    color: Scheme.text
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }
            }

            SubHeading { text: "Edition" }

            // The system's facts on the left and its logo on the right, as
            // Windows 7 shows the Windows logo beside its edition.
            Item {
                width: parent.width
                height: Math.max(editionFacts.implicitHeight, 96)

                Column {
                    id: editionFacts
                    spacing: 6

                    InfoRow { label: "Operating system"; value: systemFacts.lines[0] || "" }
                    InfoRow {
                        label: "Kernel"
                        value: systemFacts.lines[6] ? "Linux " + systemFacts.lines[6] : ""
                    }
                }

                Image {
                    anchors.right: parent.right
                    anchors.rightMargin: 24
                    anchors.verticalCenter: parent.verticalCenter
                    width: 96
                    height: 96
                    sourceSize: Qt.size(192, 192)
                    source: Qt.resolvedUrl("../assets/nixos-snowflake.svg")
                }
            }

            SubHeading { text: "System" }

            InfoRow {
                label: "Processor"
                value: (systemFacts.lines[1] || "")
                    + (systemFacts.lines[2] ? "  (" + systemFacts.lines[2] + " threads)" : "")
            }
            InfoRow {
                label: "Installed memory (RAM)"
                value: systemFacts.lines[3]
                    ? (Number(systemFacts.lines[3]) / 1048576).toFixed(1) + " GB" : ""
            }
            InfoRow {
                label: "System type"
                value: systemFacts.lines[4] === "x86_64" ? "64-bit Operating System (x86_64)"
                    : systemFacts.lines[4] || ""
            }
            InfoRow { label: "Up for"; value: systemFacts.uptime }

            SubHeading { text: "Computer name" }

            InfoRow { label: "Computer name"; value: systemFacts.lines[5] || "" }

            SubHeading { text: "This desktop" }

            InfoRow { label: "Desktop"; value: "shojiwm-win7" }
            InfoRow { label: "Compositor"; value: "ShojiWM" }
            InfoRow { label: "Shell"; value: "Quickshell" }
            InfoRow { label: "Launcher"; value: "The Start menu's search (Super+Shift+D)" }
            InfoRow { label: "User"; value: Apps.userName + " (" + Apps.loginName + ")" }
            InfoRow { label: "Settings file"; value: Settings.configHome + "/shojiwm/settings.json" }
        }
    }
}
