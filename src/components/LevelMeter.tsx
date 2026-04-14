import { useEffect, useState } from "react";
import { ipc } from "../ipc";

export default function LevelMeter() {
  const [level, setLevel] = useState(0);

  useEffect(() => {
    const id = setInterval(() => {
      ipc.getAudioLevel()
        .then(rms => setLevel(Math.min(1, Math.max(0, rms * 6))))
        .catch(() => {});
    }, 60);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="level-meter">
      <div className="level-meter-fill" style={{ width: `${(level * 100).toFixed(0)}%` }} />
    </div>
  );
}
