import QtQuick
import QtQuick.Effects
import ".."

// The orb at the left end of the taskbar, in layers as on Windows 7: the
// orb, the snowflake's glow and shadow, the NixOS snowflake, then the glass
// over them. Three states: at rest
// the snowflake glows faintly; under the pointer the orb lights up and the
// snowflake glows; while the Start menu is open both stay lit and the
// snowflake glows its brightest. A press pushes it in.
Item {
    id: orb

    property bool menuOpen: false
    signal activated
    /// A right click: the power user's menu (AdminMenu).
    signal menuRequested

    readonly property bool hovered: area.containsMouse
    readonly property bool hot: area.containsMouse || orb.menuOpen
    readonly property real pictureSize: Math.ceil(Scheme.theme.orbSize * Screen.devicePixelRatio)

    implicitWidth: Scheme.theme.orbSize
    implicitHeight: Scheme.theme.orbSize

    Image {
        anchors.fill: parent
        source: "../assets/start-orb.svg"
        sourceSize.width: orb.pictureSize
        sourceSize.height: orb.pictureSize
        smooth: true
    }

    // Lit: the same orb brighter, with a halo, faded in over the first.
    Image {
        anchors.fill: parent
        source: "../assets/start-orb-hot.svg"
        sourceSize.width: orb.pictureSize
        sourceSize.height: orb.pictureSize
        smooth: true
        opacity: orb.hot ? 1 : 0

        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) * 2 }
        }
    }

    // The glow: the snowflake blurred and tinted pale cyan, under the crisp
    // one. Faint at rest, wider and brighter under the pointer, brightest
    // while the menu is open.
    Image {
        id: glowSource

        anchors.centerIn: parent
        width: Math.round(orb.width * 0.6)
        height: width
        source: "../assets/start-orb-logo.svg"
        sourceSize.width: Math.ceil(width * Screen.devicePixelRatio)
        sourceSize.height: Math.ceil(height * Screen.devicePixelRatio)
        visible: false
    }

    MultiEffect {
        anchors.fill: glowSource
        source: glowSource
        blurEnabled: true
        blurMax: 12
        blur: 1
        colorization: 1
        colorizationColor: "#d8f6ff"
        brightness: 0.4
        scale: orb.menuOpen ? 1.3 : area.containsMouse ? 1.2 : 1.05
        opacity: orb.menuOpen ? 1 : area.containsMouse ? 0.9 : 0.35

        Behavior on scale {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) * 2 }
        }
        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) * 2 }
        }
    }

    // A soft dark shadow just under the snowflake: it sets the pale arms
    // apart from the blue glass, as the relief of the Windows flag does.
    MultiEffect {
        anchors.fill: glowSource
        anchors.verticalCenterOffset: 0.75
        source: glowSource
        blurEnabled: true
        blurMax: 4
        blur: 0.6
        colorization: 1
        colorizationColor: "#041a33"
        brightness: -0.6
        opacity: 0.7
    }

    // The snowflake itself, drawn straight from the SVG at the screen's
    // pixel size, so it is as sharp as the icons beside it.
    Image {
        anchors.centerIn: parent
        width: Math.round(orb.width * 0.6)
        height: width
        source: "../assets/start-orb-logo.svg"
        sourceSize.width: Math.ceil(width * Screen.devicePixelRatio)
        sourceSize.height: Math.ceil(height * Screen.devicePixelRatio)
        smooth: true
        scale: area.pressed ? 0.94 : 1
        opacity: orb.hot ? 1 : 0.92

        Behavior on scale {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) }
        }
    }

    // The glass: a gloss across the top half, over the snowflake.
    Image {
        anchors.fill: parent
        source: "../assets/start-orb-gloss.svg"
        sourceSize.width: orb.pictureSize
        sourceSize.height: orb.pictureSize
        smooth: true
    }

    // Pressed: a little darker, as if pushed in.
    Rectangle {
        anchors.centerIn: parent
        width: parent.width * 0.8
        height: width
        radius: width / 2
        color: "#000000"
        opacity: area.pressed ? 0.18 : 0
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: function (mouse) {
            if (mouse.button === Qt.RightButton)
                orb.menuRequested();
            else
                orb.activated();
        }
    }
}
