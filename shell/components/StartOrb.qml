import QtQuick
import QtQuick.Effects
import ".."
import "../theme.js" as Theme

// The orb at the left end of the taskbar, in layers as on Windows 7: the
// orb, the NixOS snowflake, then the glass over both. Three states: at rest
// the snowflake glows faintly; under the pointer the orb lights up and the
// snowflake glows; while the Start menu is open both stay lit and the
// snowflake glows its brightest. A press pushes it in.
Item {
    id: orb

    property bool menuOpen: false
    signal activated

    readonly property bool hot: area.containsMouse || orb.menuOpen
    readonly property real pictureSize: Math.ceil(Theme.orbSize * Screen.devicePixelRatio)

    implicitWidth: Theme.orbSize
    implicitHeight: Theme.orbSize

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
            NumberAnimation { duration: Settings.motion(Theme.fade) * 2 }
        }
    }

    Image {
        id: logo

        anchors.centerIn: parent
        width: Math.round(orb.width * 0.56)
        height: width
        source: "../assets/start-orb-logo.svg"
        sourceSize.width: Math.ceil(width * Screen.devicePixelRatio)
        sourceSize.height: Math.ceil(height * Screen.devicePixelRatio)
        smooth: true
        visible: false
    }

    // The snowflake and its glow: a shadow with no offset, in pale cyan.
    MultiEffect {
        anchors.fill: logo
        source: logo
        scale: area.pressed ? 0.94 : 1
        shadowEnabled: true
        shadowColor: "#d8f6ff"
        shadowHorizontalOffset: 0
        shadowVerticalOffset: 0
        blurMax: 12
        shadowBlur: 1
        shadowScale: orb.menuOpen ? 1.3 : area.containsMouse ? 1.2 : 1
        shadowOpacity: orb.menuOpen ? 1 : area.containsMouse ? 0.95 : 0.4
        brightness: area.pressed ? -0.15 : orb.menuOpen ? 0.3 : area.containsMouse ? 0.18 : 0
        opacity: orb.hot ? 1 : 0.9

        Behavior on shadowScale {
            NumberAnimation { duration: Settings.motion(Theme.fade) * 2 }
        }
        Behavior on shadowOpacity {
            NumberAnimation { duration: Settings.motion(Theme.fade) * 2 }
        }
        Behavior on brightness {
            NumberAnimation { duration: Settings.motion(Theme.fade) }
        }
        Behavior on scale {
            NumberAnimation { duration: Settings.motion(Theme.fade) }
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
        onClicked: orb.activated()
    }
}
