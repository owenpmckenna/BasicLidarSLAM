const canvas = document.getElementById("canvas");
const ctx = canvas.getContext('2d');
const socket = new WebSocket("ws://10.64.52.187:8081/data");
list = [];
let max_points = 7000;
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
	console.log("sending...");
	socket.send(JSON.stringify({x: x, y: y, turn: turn}));
});
document.addEventListener("keydown", (event) => {
	if (event.key == "w") {
	    x = 0.5;
	} else if (event.key === "a") {
		turn = -0.5;
	} else if (event.key === "s") {
		x = -0.5;
	} else if (event.key === "d") {
		turn = 0.5;
	}
	console.log("sending...");
	socket.send(JSON.stringify({x: x, y: y, turn: turn}));
});
function redraw() {
	ctx.fillStyle = "white";
	ctx.fillRect(0,0,600,400);
	ctx.fillStyle = "blue";
	for (m in list) {
		ctx.fillRect(list[m].x-xOffset, list[m].y-yOffset, 1, 1);
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
	redraw();
});

