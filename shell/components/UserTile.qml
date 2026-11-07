import QtQuick

// The account picture in the frame the Windows 7 log-on screen puts round
// it, measured on a Windows 7 Ultimate screenshot at 800 x 600 (116 px round
// a 98 px picture) and drawn here at 136:
//
//   a dark blue outer line, a one-pixel highlight inside it,
//   a band of glass, nearly white at the top, cyan-blue down the sides,
//   blue at the bottom (measured again on a 1280 x 720 capture),
//   a light line, then a dark grey-blue line round the picture, which is
//   square and fills its place.
//
// Each layer is one rounded rectangle inside the last, so there is one
// corner, not several.
Item {
    id: tile

    property url source

    width: 136
    height: 136

    // Everything scales with the size, so the Start menu can use it small.
    readonly property real k: width / 136
    readonly property int band: Math.max(3, Math.round(10 * k))
    readonly property real r: Math.max(3, 8 * k)

    // The shadow under the frame.
    Rectangle {
        anchors.fill: parent
        anchors.margins: -2
        anchors.topMargin: 0
        anchors.bottomMargin: -Math.max(1, Math.round(3 * tile.k))
        radius: tile.r + 2
        color: "#38001a33"
    }

    // The outer line.
    Rectangle {
        anchors.fill: parent
        radius: tile.r
        color: "#1b5f8c"
    }

    // The highlight just inside it.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: tile.r - 1
        color: "#b8ecff"
    }

    // The glass.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 2
        radius: tile.r - 2
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#e0f4f6" }
            GradientStop { position: 0.1; color: "#d2ecf0" }
            GradientStop { position: 0.35; color: "#8fc0d2" }
            GradientStop { position: 0.6; color: "#5f9fc0" }
            GradientStop { position: 0.9; color: "#3f8dbf" }
            GradientStop { position: 1.0; color: "#4a9ccf" }
        }
    }

    // The light line, then the dark one, round the picture.
    Rectangle {
        anchors.fill: parent
        anchors.margins: tile.band - 2
        radius: Math.max(1, 2 * tile.k)
        color: "#a8d8f0"
    }
    Rectangle {
        anchors.fill: parent
        anchors.margins: tile.band - 1
        color: "#476f8a"
    }

    Image {
        anchors.fill: parent
        anchors.margins: tile.band
        source: tile.source
        sourceSize.width: tile.width * 2
        sourceSize.height: tile.height * 2
        fillMode: Image.PreserveAspectCrop
        smooth: true
    }
}
