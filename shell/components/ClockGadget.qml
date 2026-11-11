import QtQuick
import Quickshell
import ".."

// The Clock gadget of the desktop: a round face of pale glass in a dark
// rim, twelve marks, the hour and minute hands and a thin red seconds hand,
// and the day under the centre. Drawn here, in the manner of the Windows 7
// gadget, not copied from it.
Item {
    id: gadget

    width: 140
    height: 140

    SystemClock {
        id: clock
        precision: SystemClock.Seconds
    }

    // A soft shadow under the face.
    Rectangle {
        x: 4
        y: 6
        width: parent.width - 4
        height: parent.height - 4
        radius: width / 2
        color: "#50000000"
    }

    readonly property real centreX: width / 2
    readonly property real centreY: height / 2
    readonly property real radius: Math.min(width, height) / 2 - 3

    // The face, painted once: the rim, the glass and the marks. Only the
    // hands move, and the GPU turns them.
    Canvas {
        anchors.fill: parent

        onPaint: {
            var context = getContext("2d");
            context.reset();
            var cx = gadget.centreX;
            var cy = gadget.centreY;
            var r = gadget.radius;

            // The rim: dark, lighter at the top.
            var rim = context.createLinearGradient(0, 0, 0, height);
            rim.addColorStop(0, "#5d6f80");
            rim.addColorStop(1, "#141c24");
            context.fillStyle = rim;
            context.beginPath();
            context.arc(cx, cy, r, 0, Math.PI * 2);
            context.fill();

            // The face: pale glass, brighter towards the top left.
            var glass = context.createRadialGradient(cx - r * 0.3, cy - r * 0.35, r * 0.1, cx, cy, r * 0.9);
            glass.addColorStop(0, "#ffffff");
            glass.addColorStop(1, "#d6e2ec");
            context.fillStyle = glass;
            context.beginPath();
            context.arc(cx, cy, r - 7, 0, Math.PI * 2);
            context.fill();

            // The marks: long at the quarters.
            context.strokeStyle = "#3a4c5c";
            context.lineCap = "round";
            for (var i = 0; i < 12; i++) {
                var angle = i * Math.PI / 6;
                var quarter = i % 3 === 0;
                var outer = r - 12;
                var inner = outer - (quarter ? 9 : 5);
                context.lineWidth = quarter ? 2.5 : 1.5;
                context.beginPath();
                context.moveTo(cx + Math.sin(angle) * inner, cy - Math.cos(angle) * inner);
                context.lineTo(cx + Math.sin(angle) * outer, cy - Math.cos(angle) * outer);
                context.stroke();
            }
        }
    }

    /// One hand: a rounded bar from a little behind the centre to its tip,
    /// turned about the centre.
    component Hand: Rectangle {
        id: hand

        property real length: 0
        property real back: 0
        /// Degrees clockwise from twelve.
        property real turn: 0

        x: gadget.centreX - width / 2
        y: gadget.centreY - length
        height: length + back
        radius: width / 2
        antialiasing: true
        transform: Rotation {
            origin.x: hand.width / 2
            origin.y: hand.length
            angle: hand.turn
        }
    }

    readonly property date now: clock.date
    readonly property real seconds: now.getSeconds()
    readonly property real minutes: now.getMinutes() + seconds / 60
    readonly property real hours: (now.getHours() % 12) + minutes / 60

    Hand {
        length: gadget.radius * 0.45
        back: 6
        width: 4.5
        color: "#1f2b36"
        turn: gadget.hours * 30
    }

    Hand {
        length: gadget.radius * 0.68
        back: 8
        width: 3
        color: "#1f2b36"
        turn: gadget.minutes * 6
    }

    Hand {
        length: gadget.radius * 0.74
        back: 12
        width: 1.2
        color: "#c42b1c"
        turn: gadget.seconds * 6
    }

    Rectangle {
        x: gadget.centreX - 3
        y: gadget.centreY - 3
        width: 6
        height: 6
        radius: 3
        color: "#c42b1c"
    }

    // The gloss over the upper half of the glass, painted once.
    Canvas {
        anchors.fill: parent

        onPaint: {
            var context = getContext("2d");
            context.reset();
            var cx = gadget.centreX;
            var cy = gadget.centreY;
            var r = gadget.radius;
            var gloss = context.createLinearGradient(0, cy - r, 0, cy);
            gloss.addColorStop(0, "rgba(255, 255, 255, 0.55)");
            gloss.addColorStop(1, "rgba(255, 255, 255, 0)");
            context.fillStyle = gloss;
            context.beginPath();
            context.ellipse(cx - (r - 12), cy - r + 9, (r - 12) * 2, r * 0.9);
            context.fill();
        }
    }

    Text {
        anchors.horizontalCenter: parent.horizontalCenter
        y: parent.height * 0.64
        text: Qt.formatDate(clock.date, "ddd d")
        color: "#4d6578"
        font.family: Scheme.theme.fontFamily
        font.pixelSize: 10
    }
}
