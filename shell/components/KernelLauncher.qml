import QtQuick
import ".."

// The Kernel theme's Start button, at the left end of the bar: a plate in the
// steel blue of the window captions, as tall as the bar, with the NixOS
// snowflake in white. Lighter under the pointer, darker while the menu is
// open or the button held. It has the orb's interface (StartOrb.qml).
Item {
    id: launcher

    property bool menuOpen: false
    signal activated
    /// A right click: the power user's menu (AdminMenu).
    signal menuRequested

    implicitWidth: Scheme.theme.orbSize
    implicitHeight: Scheme.theme.taskbarHeight

    readonly property bool sunk: area.pressed || launcher.menuOpen

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop {
                position: 0.0
                color: launcher.sunk ? Scheme.theme.buttonPressedTop
                    : area.containsMouse ? Qt.lighter(Scheme.theme.buttonActiveTop, 1.15)
                    : Scheme.theme.buttonActiveTop
            }
            GradientStop {
                position: 1.0
                color: launcher.sunk ? Scheme.theme.buttonPressedBottom
                    : area.containsMouse ? Qt.lighter(Scheme.theme.buttonActiveBottom, 1.15)
                    : Scheme.theme.buttonActiveBottom
            }
        }
    }

    // The line of light along its top (none while sunk) and the dark seam on
    // its right, as a caption button's plate.
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 1
        color: Scheme.theme.barHighlight
        visible: !launcher.sunk
    }
    Rectangle {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        width: 1
        color: Scheme.theme.barEdge
    }

    Image {
        anchors.centerIn: parent
        width: Math.round(launcher.height * 0.56)
        height: width
        source: "../assets/start-orb-logo.svg"
        sourceSize.width: Math.ceil(width * Screen.devicePixelRatio)
        sourceSize.height: Math.ceil(height * Screen.devicePixelRatio)
        smooth: true
        opacity: launcher.sunk ? 0.85 : 1
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: function (mouse) {
            if (mouse.button === Qt.RightButton)
                launcher.menuRequested();
            else
                launcher.activated();
        }
    }
}
