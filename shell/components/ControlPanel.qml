pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
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
        "appearance": { title: "Style, light or dark", category: "Appearance and Personalization" },
        "background": { title: "Desktop Background", category: "Appearance and Personalization" },
        "lockscreen": { title: "Lock Screen", category: "Appearance and Personalization" },
        "account": { title: "Account Picture", category: "User Accounts" },
        "taskbar": { title: "Taskbar and Start Menu", category: "Appearance and Personalization" },
        "clock": { title: "Date and Time", category: "Clock and Region" },
        "about": { title: "System", category: "System" }
    })

    function go(page) {
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
            radius: Scheme.modern ? 0 : 13
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

        // The path: Control Panel > category > page.
        Rectangle {
            anchors.left: back.right
            anchors.leftMargin: 8
            anchors.right: parent.right
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
                        "taskbar", "clock", "about"]

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

        Loader {
            id: content

            x: 28
            y: 22
            width: flick.width - 56
            sourceComponent: {
                switch (window.page) {
                case "appearance": return appearancePage;
                case "background": return backgroundPage;
                case "lockscreen": return lockPage;
                case "account": return accountPage;
                case "taskbar": return taskbarPage;
                case "clock": return clockPage;
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
                        { text: "Choose a style, light or dark", page: "appearance" },
                        { text: "Change desktop background", page: "background" },
                        { text: "Taskbar and Start menu", page: "taskbar" }
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

            Heading { text: "Choose a style" }

            Note {
                width: parent.width
                text: "Aero is Windows 7 as it was. Modern is the same Aero, "
                    + "squared: the glass, the gloss and the colours stay, and "
                    + "every corner, of the windows, the menus, the buttons and "
                    + "the lock screen, is a right angle."
            }

            Row {
                spacing: 18

                Repeater {
                    model: [
                        { value: "aero", label: "Aero" },
                        { value: "modern", label: "Modern" }
                    ]

                    delegate: Item {
                        id: styleChoice

                        required property var modelData
                        readonly property bool isChosen: Settings.style === modelData.value
                        readonly property bool isModern: modelData.value === "modern"

                        width: 200
                        height: 150

                        Rectangle {
                            anchors.fill: parent
                            radius: Scheme.itemRadius
                            color: styleChoice.isChosen ? Scheme.selection
                                : styleArea.containsMouse ? Scheme.hover : "transparent"
                            border.width: styleChoice.isChosen || styleArea.containsMouse
                                ? Scheme.selectionBorderWidth : 0
                            border.color: Scheme.selectionBorder
                        }

                        // A small Aero window on a bit of desktop, rounded or
                        // square.
                        Rectangle {
                            x: 16
                            y: 12
                            width: parent.width - 32
                            height: 96
                            gradient: Gradient {
                                GradientStop { position: 0.0; color: "#1b5e97" }
                                GradientStop { position: 1.0; color: "#2f8cc4" }
                            }

                            Rectangle {
                                x: 14
                                y: 10
                                width: parent.width - 28
                                height: 78
                                radius: styleChoice.isModern ? 0 : 5
                                color: "#b3538ebe"
                                border.width: 1
                                border.color: "#000000"

                                Rectangle {
                                    anchors.fill: parent
                                    anchors.margins: 1
                                    radius: styleChoice.isModern ? 0 : 4
                                    color: "transparent"
                                    border.width: 1
                                    border.color: "#ccffffff"
                                }

                                // The caption buttons.
                                Row {
                                    anchors.right: parent.right
                                    anchors.rightMargin: 4
                                    y: 1

                                    Repeater {
                                        model: 3
                                        delegate: Rectangle {
                                            required property int index
                                            width: index === 2 ? 20 : 12
                                            height: 9
                                            radius: styleChoice.isModern ? 0 : 1
                                            color: index === 2 ? "#c94535" : "#9fc4e3"
                                            border.width: 1
                                            border.color: "#66000000"
                                        }
                                    }
                                }

                                Rectangle {
                                    x: 5
                                    y: 16
                                    width: parent.width - 10
                                    height: parent.height - 21
                                    color: "#ffffff"
                                    border.width: 1
                                    border.color: "#8c0b1b2a"
                                }
                            }
                        }

                        CpRadio {
                            anchors.horizontalCenter: parent.horizontalCenter
                            anchors.bottom: parent.bottom
                            anchors.bottomMargin: 10
                            label: styleChoice.modelData.label
                            checked: styleChoice.isChosen
                            onClicked: Settings.style = styleChoice.modelData.value
                        }

                        MouseArea {
                            id: styleArea
                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: Settings.style = styleChoice.modelData.value
                        }
                    }
                }
            }

            Item { width: 1; height: 8 }

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
                    + "chosen here, ~/.face is used, the picture log-in screens read."
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
