const canvas = document.getElementById("canvas");
const ctx = canvas.getContext('2d');
const socket = new WebSocket("ws://127.0.0.1:8081/data");
list = [];
let xOffset = -300;
let yOffset = -200;
document.addEventListener("keydown", (event) => {
	if (event.key == "ArrowUp") {
		yOffset += 5;
	} else if (event.key === "ArrowDown") {
		yOffset -= 5;
	} else if (event.key === "ArrowLeft") {
		xOffset -= 5;
	} else if (event.key === "ArrowRight") {
		xOffset += 5;
	}
});
function redraw() {
	ctx.fillStyle = "white";
	ctx.fillRect(0,0,600,400);
	ctx.fillStyle = "blue";
	for (x in list) {
		ctx.fillRect(x.x-xOffset, x.y-yOffset, 1, 1);
	}
}
socket.addEventListener("message", (event) => {
	const data = JSON.parse(event.data);
	console.log(data);
	for (msg in data.data) {
	    console.log(msg);
		list.push({x: msg.x, y: msg.y});
	}
	redraw();
});

