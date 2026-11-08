import QtQuick
import "../theme.js" as Theme

// The orb at the left end of the taskbar. It lights up under the pointer and
// stays lit while the Start menu is open.
Item {
    id: orb

    property bool menuOpen: false
    signal activated

    implicitWidth: Theme.orbSize
    implicitHeight: Theme.orbSize

    Image {
        id: picture
        anchors.fill: parent
        source: "../assets/start-orb.svg"
        sourceSize.width: Theme.orbSize * 2
        sourceSize.height: Theme.orbSize * 2
        smooth: true
    }

    // Lit: the same orb brighter, with a halo, faded in over the first.
    Image {
        anchors.fill: parent
        source: "../assets/start-orb-hot.svg"
        sourceSize.width: Theme.orbSize * 2
        sourceSize.height: Theme.orbSize * 2
        smooth: true
        opacity: orb.menuOpen || area.containsMouse ? 1 : 0

        Behavior on opacity {
            NumberAnimation { duration: Theme.fade * 2 }
        }
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
