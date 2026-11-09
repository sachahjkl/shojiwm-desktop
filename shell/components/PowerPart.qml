import QtQuick
import ".."
import "../theme.js" as Theme

// One half of the Start menu's power button: the button itself, or the
// arrow beside it. The two halves share an edge and round only their outer
// corners.
Item {
    id: part

    property bool leftRounded: false
    property bool rightRounded: false
    property bool hot: false
    property bool pressed: false
    /// On the Start menu's glass: a pale band of glass with a light rim,
    /// for white text.
    property bool glass: false

    height: parent ? parent.height : 28

    readonly property color outline: part.glass ? "#a6000000" : Scheme.menuOutline

    readonly property color fillTop: part.glass
        ? (part.pressed ? "#26ffffff" : part.hot ? "#66ffffff" : "#40ffffff")
        : part.pressed ? Scheme.buttonPressedTop
            : part.hot ? Scheme.buttonHotTop : Scheme.buttonTop
    readonly property color fillBottom: part.glass
        ? (part.pressed ? "#14ffffff" : part.hot ? "#38ffffff" : "#1fffffff")
        : part.pressed ? Scheme.buttonPressedBottom
            : part.hot ? Scheme.buttonHotBottom : Scheme.buttonBottom

    // A rectangle rounded all round, wider than the half on its inner side
    // and clipped there: the inner corners fall outside, and the glass, which
    // is translucent, is painted once.
    Item {
        anchors.fill: parent
        clip: part.leftRounded !== part.rightRounded

        Rectangle {
            readonly property real overhang: part.leftRounded === part.rightRounded
                ? 0 : Scheme.buttonRadius + 1

            x: part.rightRounded && !part.leftRounded ? -overhang : 0
            width: parent.width + overhang
            height: parent.height
            radius: Scheme.buttonRadius
            border.width: 1
            border.color: part.outline
            gradient: Gradient {
                GradientStop { position: 0.0; color: part.fillTop }
                GradientStop { position: 1.0; color: part.fillBottom }
            }
        }
    }

    // The seam between the two halves, on the arrow's left edge: one dark
    // line, and on glass a faint light one beside it.
    Rectangle {
        visible: part.rightRounded && !part.leftRounded
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 1
        color: part.outline
    }
    Rectangle {
        visible: part.glass && part.rightRounded && !part.leftRounded
        x: 1
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.topMargin: 1
        anchors.bottomMargin: 1
        width: 1
        color: "#26ffffff"
    }
}
