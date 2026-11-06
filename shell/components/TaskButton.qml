import QtQuick
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// One window on the taskbar: its icon, its title, and the state of the
// window it stands for. A left click shows the window or puts it away, a
// right click closes it.
Rectangle {
    id: button

    required property var toplevel
    /// Set by the taskbar while the Alt+Tab panel or anything else is up.
    property bool pressedDown: area.pressed

    readonly property bool current: toplevel
        && toplevel.activated && !toplevel.minimized
    readonly property bool hovered: area.containsMouse

    height: Theme.taskButtonHeight
    radius: 3
    antialiasing: true

    border.width: 1
    border.color: current ? Theme.buttonActiveBorder
        : hovered ? Theme.buttonHoverBorder : Theme.buttonRestBorder

    readonly property color fill: pressedDown ? Theme.buttonActive
        : current ? Theme.buttonActive
        : hovered ? Theme.buttonHover : Theme.buttonRest

    gradient: Gradient {
        GradientStop { position: 0.0; color: Theme.buttonGloss }
        GradientStop { position: 0.48; color: button.fill }
        GradientStop { position: 0.52; color: Qt.darker(button.fill, 1.15) }
        GradientStop { position: 1.0; color: button.fill }
    }

    Behavior on border.color {
        ColorAnimation { duration: Theme.fade }
    }

    Row {
        anchors.fill: parent
        anchors.leftMargin: 6
        anchors.rightMargin: 6
        spacing: 6

        IconImage {
            anchors.verticalCenter: parent.verticalCenter
            implicitSize: 16
            source: Apps.iconFor(button.toplevel ? button.toplevel.appId : "")
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(0, button.width - 12 - 16 - 6)
            text: Apps.titleOf(button.toplevel)
            color: Theme.textOnGlass
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            elide: Text.ElideRight
            visible: button.width > 70
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onClicked: function (event) {
            if (event.button === Qt.RightButton) {
                if (button.toplevel)
                    button.toplevel.close();
                return;
            }
            Apps.toggleWindow(button.toplevel);
        }
    }
}
