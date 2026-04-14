import { BrowserRouter, Routes, Route, NavLink, Navigate } from "react-router-dom";
import Profiles from "./pages/Profiles";
import Providers from "./pages/Providers";
import Vocabulary from "./pages/Vocabulary";
import Audio from "./pages/Audio";
import Models from "./pages/Models";
import General from "./pages/General";
import History from "./pages/History";

export default function App() {
  return (
    <BrowserRouter>
      <div className="app">
        <nav className="sidebar">
          <h2>Dictatr</h2>
          <NavLink to="/profiles">Profile</NavLink>
          <NavLink to="/providers">LLM-Anbieter</NavLink>
          <NavLink to="/vocabulary">Wörterbuch</NavLink>
          <NavLink to="/audio">Audio</NavLink>
          <NavLink to="/models">Modelle</NavLink>
          <NavLink to="/general">Allgemein</NavLink>
          <NavLink to="/history">History</NavLink>
        </nav>
        <main className="content">
          <Routes>
            <Route path="/" element={<Navigate to="/profiles" />} />
            <Route path="/profiles" element={<Profiles />} />
            <Route path="/providers" element={<Providers />} />
            <Route path="/vocabulary" element={<Vocabulary />} />
            <Route path="/audio" element={<Audio />} />
            <Route path="/models" element={<Models />} />
            <Route path="/general" element={<General />} />
            <Route path="/history" element={<History />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}
