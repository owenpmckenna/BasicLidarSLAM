const canvas = document.getElementById("canvas");
const ctx = canvas.getContext('2d');
const serveraddr = "10.64.52.126:8081";
const socket = new WebSocket("ws://" + serveraddr + "/data");
list = [];
let max_points = 600;
let number = 0;
for (let i = 0; i < max_points; i += 1) {
    list.push({x: 0, y: 0});
}
let xOffset = -300;
let yOffset = -200;
document.addEventListener("keydown", (event) => {
	if (event.key == "ArrowUp") {
		yOffset -= 5;
	} else if (event.key === "ArrowDown") {
		yOffset += 5;
	} else if (event.key === "ArrowLeft") {
		xOffset += 5;
	} else if (event.key === "ArrowRight") {
		xOffset -= 5;
	}
});
let x = 0.0;
let y = 0.0;
let turn = 0.0;
let speed = 0.2;
document.addEventListener("keyup", (event) => {
	if (event.key == "w") {
	    x = 0.0;
	} else if (event.key === "a") {
		turn = 0.0;
	} else if (event.key === "s") {
		x = 0.0;
	} else if (event.key === "d") {
		turn = 0.0;
	}
	console.log("sending up...");
	fetch("http://" + serveraddr + "/motorcontrol", {
        method: 'POST',
        body: JSON.stringify({x: x, y: y, turn: turn}),
        headers: {
            "Content-Type": "application/json",
        },
	}).then((response) => response.json());
	//socket.send(JSON.stringify({x: x, y: y, turn: turn}));
});
document.addEventListener("keydown", (event) => {
    if (event.repeat) return;
	if (event.key == "w") {
	    x = speed;
	} else if (event.key === "a") {
		turn = speed;
	} else if (event.key === "s") {
		x = -speed;
	} else if (event.key === "d") {
		turn = -speed;
	}
	console.log("sending down...");
	//socket.send(JSON.stringify({x: x, y: y, turn: turn}));
	fetch("http://" + serveraddr + "/motorcontrol", {
        method: 'POST',
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({x: x, y: y, turn: turn})
	}).then((response) => response.json());
});
function redraw(lines) {
	ctx.fillStyle = "white";
	ctx.fillRect(0,0,600,400);
	ctx.fillStyle = "blue";
	for (m in list) {
		ctx.fillRect(list[m].x-xOffset, list[m].y-yOffset, 1, 1);
	}
	//if (lines.length != 0) {
	    console.log(lines)
	//}
	for (m in lines) {
	    ctx.fillStyle = "green";
	    ctx.fillRect(lines[m].mid[0] - xOffset - 2, lines[m].mid[1] - yOffset - 2, 4, 4);
	    //ctx.fillStyle = "yellow";
	    //ctx.fillRect(lines[m].mid[0] - xOffset - 2, lines[m].mid[1] - yOffset - 2, lines[m].length, 4);
	    ctx.strokeStyle = "orange";
	    ctx.lineWidth = 3;
	    ctx.beginPath();
        ctx.moveTo(lines[m].p0[0] - xOffset, lines[m].p0[1] - yOffset);
        ctx.lineTo(lines[m].p1[0] - xOffset, lines[m].p1[1] - yOffset);
        ctx.stroke();
	}
	ctx.fillStyle = "red";
	ctx.fillRect(-2 - xOffset, -2 - yOffset, 4, 4);
}
socket.addEventListener("message", (event) => {
	const data = JSON.parse(event.data);
	console.log(data);
	data.data.forEach(msg => {
	    list[number % max_points] = ({x: msg.x, y: msg.y});
	    number += 1;
	});
	redraw(data.lines);
});

