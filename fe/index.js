import * as rtls from "rtls";

console.log("start");

const dev_canvas = document.getElementById("device-map");
dev_canvas.height = 640
dev_canvas.width = 640
const grid_resolution = 50

const ctx = dev_canvas.getContext('2d');

const zone = rtls.init();
zone.add_device(1, 15, 30, 0);
zone.add_device(2, 300, 270, 0);
zone.add_measure(1, 2, 200, 123);
zone.add_measure(1, 2, 220, 127);

const drawGrid = () => {
        ctx.beginPath();
        ctx.strokeStyle = "#CCCCCC";
        ctx.font = "12px Arial";
        ctx.textAlign = "left";
        // draw grid
        for (let col = 0; col < dev_canvas.width; col += grid_resolution) {
                ctx.moveTo(col, 0);
                ctx.lineTo(col, dev_canvas.height);
                ctx.fillText(col, col, dev_canvas.height - 6);
        }
        for (let row = 0; row < dev_canvas.height; row += grid_resolution) {
                ctx.moveTo(0, row);
                ctx.lineTo(dev_canvas.width, row);
                ctx.fillText(row, 0, dev_canvas.height - row + 6);
        }
        // Draw border
        ctx.moveTo(0, 0);
        ctx.lineTo(dev_canvas.width, 0);
        ctx.lineTo(dev_canvas.width, dev_canvas.height);
        ctx.lineTo(0, dev_canvas.height);
        ctx.lineTo(0, 0);

        ctx.stroke();
}

function get_img(id)
{
        if (typeof get_img.list == 'undefined') {
                get_img.list = [];
                let path = "/icons/avatars/";
                var names = ["m1.svg", "m2.svg", "w1.svg", "w2.svg"];
                names.forEach((elem) => {
                        let img = new Image();
                        img.src = path + elem;
                        get_img.list.push(img);
                });
        }
        return get_img.list[id % get_img.list.length];
}

function posToPixels(pos) {
        return [pos[0], dev_canvas.height-pos[1]];
}

const drawDevices = () => {
        const SIZE = 50
        var date = new Date();
        let devs = zone.get_all_devices_position(date.getTime());
        ctx.fillStyle = "#121540";
        ctx.font = "25px Arial";
        ctx.textAlign = "center";

        devs.forEach((elem) => {
                let pos = posToPixels(elem.pos.cord);
                ctx.drawImage(get_img(elem.id), pos[0] - SIZE / 2, pos[1] - SIZE, SIZE, SIZE);
                ctx.fillText(elem.id, pos[0], pos[1] + 25);
        });

        ctx.stroke()
}

const renderLoop = () => {
        ctx.clearRect(0, 0, dev_canvas.width, dev_canvas.height); // clear canvas
        drawGrid();
        drawDevices()

        zone.add_measure(1, 2, 220, 127);
        zone.add_measure(1, 0, 220, 127);
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