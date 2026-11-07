import QtQuick

// The red power button of the Windows 7 log-on screen, and the arrow joined
// to its right that opens the other choices. Measured at 38 + 14 x 18 px at
// 800 x 600: one rounded piece with a dark outline and a dark line between
// the two parts, a pale red gloss over the top half, a deep red below that
// brightens at the bottom, and a white ring for the power mark.
Item {
    id: split

    signal power
    signal menu
    /// The modern style: two flat translucent circles, the power mark and
    /// the arrow, in place of the red gloss.
    property bool modern: false

    width: modern ? 36 + 4 + 24 : 70
    height: modern ? 36 : 24

    readonly property int divider: 50

    // One part's fill: lit under the pointer, darker while pressed.
    component Fill: Rectangle {
        property bool hot: false
        property bool down: false

        gradient: Gradient {
            GradientStop { position: 0.0; color: down ? "#a65454" : hot ? "#eda0a0" : "#d98585" }
            GradientStop { position: 0.47; color: down ? "#8c2c2c" : hot ? "#cf5c5c" : "#b84a4a" }
            GradientStop { position: 0.5; color: down ? "#5e0000" : hot ? "#a30000" : "#860000" }
            GradientStop { position: 1.0; color: down ? "#8a0505" : hot ? "#e01515" : "#c00808" }
        }
    }

    // The outline, and the clip that rounds both parts as one.
    Rectangle {
        id: shape
        visible: !split.modern
        anchors.fill: parent
        radius: 3
        color: "#2a0303"
        clip: true

        Fill {
            x: 1
            y: 1
            width: split.divider - 1
            height: parent.height - 2
            radius: 2
            hot: mainArea.containsMouse
            down: mainArea.pressed
        }

        Fill {
            x: split.divider + 1
            y: 1
            width: parent.width - split.divider - 2
            height: parent.height - 2
            radius: 2
            hot: arrowArea.containsMouse
            down: arrowArea.pressed
        }
    }

    // The light rim inside the outline.
    Rectangle {
        visible: !split.modern
        anchors.fill: parent
        anchors.margins: 1
        radius: 2
        color: "transparent"
        border.width: 1
        border.color: "#4dffdada"
    }

    Image {
        visible: !split.modern
        x: (split.divider - width) / 2
        anchors.verticalCenter: parent.verticalCenter
        width: 16
        height: 16
        source: "../assets/power-ring.svg"
        sourceSize.width: 32
        sourceSize.height: 32
    }

    Text {
        visible: !split.modern
        x: split.divider + (split.width - split.divider - width) / 2
        anchors.verticalCenter: parent.verticalCenter
        text: "\u25B2"
        color: "#ffffff"
        font.pixelSize: 7
    }

    // ----- modern -----------------------------------------------------------

    Rectangle {
        visible: split.modern
        width: 36
        height: 36
        radius: 18
        color: mainArea.pressed ? "#1fffffff" : mainArea.containsMouse ? "#47ffffff" : "#29ffffff"
        border.width: 1
        border.color: "#33ffffff"

        Image {
            anchors.centerIn: parent
            width: 16
            height: 16
            source: "../assets/power-white.svg"
            sourceSize.width: 32
            sourceSize.height: 32
        }
    }

    Rectangle {
        visible: split.modern
        x: 40
        anchors.verticalCenter: parent.verticalCenter
        width: 24
        height: 24
        radius: 12
        color: arrowArea.pressed ? "#1fffffff" : arrowArea.containsMouse ? "#47ffffff" : "#1affffff"

        Text {
            anchors.centerIn: parent
            text: "\u25B2"
            color: "#ffffff"
            font.pixelSize: 7
        }
    }

    MouseArea {
        id: mainArea
        width: split.modern ? 36 : split.divider
        height: parent.height
        hoverEnabled: true
        onClicked: split.power()
    }

    MouseArea {
        id: arrowArea
        x: split.modern ? 40 : split.divider
        width: split.modern ? 24 : split.width - split.divider
        height: parent.height
        hoverEnabled: true
        onClicked: split.menu()
    }
}
