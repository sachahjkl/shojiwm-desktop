import QtQuick
import ".."
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
        source: Scheme.modern ? "../assets/start-modern.svg" : "../assets/start-orb.svg"
        sourceSize.width: Theme.orbSize * 2
        sourceSize.height: Theme.orbSize * 2
        smooth: true
    }

    // The highlight: a pale wash over the orb, kept inside its circle.
    Rectangle {
        anchors.centerIn: parent
        width: Theme.orbSize - 6
        height: width
        radius: Scheme.modern ? 7 : width / 2
        color: "#ffffff"
        opacity: orb.menuOpen ? 0.3 : area.containsMouse ? 0.18 : 0

        Behavior on opacity {
            NumberAnimation { duration: Theme.fade }
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: orb.activated()
    }
}
