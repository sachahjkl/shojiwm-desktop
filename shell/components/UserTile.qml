import QtQuick

// The account picture in the frame the Windows 7 log-on screen puts round
// it: a rounded square of pale blue glass, lit at the top, a dark line
// outside, a dark line round the picture, and a soft shadow under it.
// Measured at 116 px round a 98 px picture at 800 x 600; drawn here at 136
// round 114.
Item {
    id: tile

    property url source

    width: 136
    height: 136

    // The shadow.
    Rectangle {
        anchors.fill: parent
        anchors.margins: -3
        anchors.topMargin: -1
        radius: 13
        color: "#33000000"
    }

    // The glass.
    Rectangle {
        anchors.fill: parent
        radius: 12
        border.width: 1
        border.color: "#1d5580"
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#c9ebfb" }
            GradientStop { position: 0.45; color: "#8fcdee" }
            GradientStop { position: 0.55; color: "#73b8e2" }
            GradientStop { position: 1.0; color: "#4892c6" }
        }
    }

    // The lit inner edge of the glass.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: 11
        color: "transparent"
        border.width: 1
        border.color: "#8cffffff"
    }

    // The picture, in its own dark line.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 8
        radius: 4
        color: "#ffffff"
        border.width: 1
        border.color: "#4d6f8a"
        clip: true

        Image {
            anchors.fill: parent
            anchors.margins: 1
            source: tile.source
            sourceSize.width: 228
            sourceSize.height: 228
            fillMode: Image.PreserveAspectCrop
            smooth: true
        }

        // The gloss over the top of the picture.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 1
            height: parent.height * 0.45
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#40ffffff" }
                GradientStop { position: 1.0; color: "#00ffffff" }
            }
        }
    }
}
