import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export default function LevelMeter() {
  const [level, setLevel] = useState(0);

  useEffect(() => {
    const unlistenPromise = listen<number>("audio://level", (e) => {
      setLevel(Math.min(1, Math.max(0, e.payload * 4)));
    });
    return () => { unlistenPromise.then((u) => u()); };
  }, []);

  return (
    <div className="level-meter">
      <div className="level-meter-fill" style={{ width: `${(level * 100).toFixed(0)}%` }} />
    </div>
  );
}
