const fs = require("fs");

async function main() {
  const clientId = process.argv[2];
  const pubKeyPath = process.argv[3];
  const apiUrl = process.argv[4] || "http://127.0.0.1:3000/api/tunnel-key";

  if (!clientId || !pubKeyPath) {
    console.error("Usage: node send_key.js CLIENT_ID /path/to/key.pub [API_URL]");
    process.exit(2);
  }

  const publicKey = fs.readFileSync(pubKeyPath, "utf8").trim();

  const res = await fetch(apiUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ clientId, publicKey }),
  });

  if (!res.ok) {
    const text = await res.text().catch(() => "");
    console.error(`HTTP ${res.status} ${res.statusText} ${text}`.trim());
    process.exit(1);
  }

  console.log("c'est good envoyé");
}

main().catch((e) => {
  console.error("nn ça marche pas", e.message);
  process.exit(1);
});
