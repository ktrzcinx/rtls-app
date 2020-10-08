import * as rtls from "rtls";

console.log("start");

const dev_canvas = document.getElementById("device-map");
dev_canvas.height = 640
dev_canvas.width = 640
const grid_resolution = 50

const ctx = dev_canvas.getContext('2d');

const zone = rtls.init();
zone.add_device(1, 15, 30, 0);
zone.add_device(2, 30, 40, 0);
zone.add_measure(1, 2, 200, 123);
zone.add_measure(1, 2, 220, 127);

const drawGrid = () => {
        ctx.beginPath();
        ctx.strokeStyle = "#CCCCCC";
        // draw grid
        for (let col = 0; col < dev_canvas.width; col += grid_resolution) {
                ctx.moveTo(col, 0);
                ctx.lineTo(col, dev_canvas.height);
        }
        for (let row = 0; row < dev_canvas.height; row += grid_resolution) {
                ctx.moveTo(0, row);
                ctx.lineTo(dev_canvas.width, row);
        }
        // Draw border
        ctx.moveTo(0, 0);
        ctx.lineTo(dev_canvas.width, 0);
        ctx.lineTo(dev_canvas.width, dev_canvas.height);
        ctx.lineTo(0, dev_canvas.height);
        ctx.lineTo(0, 0);

        ctx.stroke();
}

const drawDevices = () => {
        const SIZE = 10
        let pos = zone.calculate_device_position(1, 15);
        console.log(pos.cord);
        
        ctx.beginPath();
        ctx.fillStyle = "#BBCC00";
        ctx.rect(pos.cord[0] - SIZE / 2, pos.cord[1] - SIZE / 2, SIZE, SIZE);
        ctx.stroke()
}

const renderLoop = () => {
        drawGrid();
        drawDevices()

        ctx.beginPath();
        ctx.fillStyle = "#000000";
        ctx.fillRect(10, 10, 20, 30);
        ctx.stroke()

        requestAnimationFrame(renderLoop);
};

const test_code = () => {
        let dev_ptr = zone.get_device_ptr(0);
        let dev = rtls.device_serialize(dev_ptr);
        console.log('dev:', dev);
        let pos = dev.trace[0].cord
        console.log(pos)
}

test_code()
renderLoop()