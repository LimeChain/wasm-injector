const fs = require("fs");
const file = fs.readFileSync(process.argv[2]);
const buff = Buffer.from(file);
console.log(`0x${buff.toString("hex")}`);
