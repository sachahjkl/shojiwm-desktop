pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls.Basic
import Quickshell
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
        "appearance": { title: "Light or dark", category: "Appearance and Personalization" },
        "background": { title: "Desktop Background", category: "Appearance and Personalization" },
        "lockscreen": { title: "Lock Screen", category: "Appearance and Personalization" },
        "account": { title: "Account Picture", category: "User Accounts" },
        "taskbar": { title: "Taskbar and Start Menu", category: "Appearance and Personalization" },
        "notifications": { title: "Notification Area Icons", category: "Appearance and Personalization" },
        "clock": { title: "Date and Time", category: "Clock and Region" },
        "ease": { title: "Ease of Access", category: "Ease of Access" },
        "about": { title: "System", category: "System" }
    })

    // ----- search -------------------------------------------------------------
    //
    // Every task the pages hold, with the words it is found by. Typing in the
    // search box replaces the page with the tasks that match every word.

    readonly property var tasks: [
        { text: "Choose light or dark", page: "appearance", words: "dark light theme colour color scheme night appearance" },
        { text: "Change desktop background", page: "background", words: "wallpaper picture image desktop background" },
        { text: "Change the size of desktop icons", page: "taskbar", words: "desktop icons size large small medium zoom" },
        { text: "Lock screen and its picture", page: "lockscreen", words: "lock screen picture password idle delay suspend sleep" },
        { text: "Lock the computer after a delay", page: "lockscreen", words: "lock idle minutes delay timeout screensaver" },
        { text: "Change your account picture", page: "account", words: "account user picture avatar face photo profile" },
        { text: "Combine taskbar buttons", page: "taskbar", words: "taskbar combine group buttons windows" },
        { text: "Show labels on taskbar buttons", page: "taskbar", words: "taskbar labels compact icons text" },
        { text: "Use Aero Peek to preview the desktop", page: "taskbar", words: "aero peek preview desktop show" },
        { text: "Show the programs used most in the Start menu", page: "taskbar", words: "start menu frequent recent programs" },
        { text: "Change what the power button does", page: "taskbar", words: "power button shut down shutdown restart sleep hibernate log off lock" },
        { text: "Select which icons appear on the taskbar", page: "notifications", words: "notification area tray icons hidden show" },
        { text: "Change the clock format", page: "clock", words: "clock time date format 24 12 hour" },
        { text: "Turn off all unnecessary animations", page: "ease", words: "animations motion reduce reduced effects ease access accessibility" },
        { text: "About this desktop", page: "about", words: "about system version information" }
    ]

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
                    source: "../assets/icons/control-panel.svg"
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
                model: ["appearance", "background", "lockscreen", "account",
                        "taskbar", "notifications", "clock", "ease", "about"]

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
            source: "../assets/icons/" + category.icon + ".svg"
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
                    + "(GTK, libadwaita, Qt, Firefox, Chromium, Vicinae) change "
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
                label: "Use Aero Peek to preview the desktop"
                checked: Settings.peek
                onToggled: function (on) { Settings.peek = on; }
            }

            SubHeading { text: "Start menu" }

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
        }
    }

    Component {
        id: aboutPage

        Column {
            spacing: 10

            Heading { text: "About this desktop" }

            Repeater {
                model: [
                    ["Desktop", "shojiwm-win7"],
                    ["Compositor", "ShojiWM"],
                    ["Shell", "Quickshell"],
                    ["Launcher", "Vicinae (Super+D)"],
                    ["User", Apps.userName + " (" + Apps.loginName + ")"],
                    ["Settings file", Settings.configHome + "/shojiwm/settings.json"]
                ]

                delegate: Row {
                    required property var modelData
                    spacing: 12

                    Text {
                        width: 120
                        text: modelData[0]
                        color: Scheme.textMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }
                    Text {
                        text: modelData[1]
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }
                }
            }
        }
    }
}
