import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

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

    Canvas {
        id: face

        anchors.fill: parent
        property date now: clock.date
        onNowChanged: requestPaint()

        onPaint: {
            var context = getContext("2d");
            context.reset();
            var cx = width / 2;
            var cy = height / 2;
            var r = Math.min(width, height) / 2 - 3;

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

            var t = face.now;
            var seconds = t.getSeconds();
            var minutes = t.getMinutes() + seconds / 60;
            var hours = (t.getHours() % 12) + minutes / 60;
            function hand(angle, length, widthPx, colour, back) {
                context.strokeStyle = colour;
                context.lineWidth = widthPx;
                context.beginPath();
                context.moveTo(cx - Math.sin(angle) * back, cy + Math.cos(angle) * back);
                context.lineTo(cx + Math.sin(angle) * length, cy - Math.cos(angle) * length);
                context.stroke();
            }
            hand(hours * Math.PI / 6, r * 0.45, 4.5, "#1f2b36", 6);
            hand(minutes * Math.PI / 30, r * 0.68, 3, "#1f2b36", 8);
            hand(seconds * Math.PI / 30, r * 0.74, 1.2, "#c42b1c", 12);

            context.fillStyle = "#c42b1c";
            context.beginPath();
            context.arc(cx, cy, 3, 0, Math.PI * 2);
            context.fill();

            // The gloss over the upper half of the glass.
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
        font.family: Theme.fontFamily
        font.pixelSize: 10
    }
}
