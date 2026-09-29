import {
  CalendarDays,
  CheckSquare2,
  CircleGauge,
  Goal,
  Landmark,
  Leaf,
  Settings,
} from 'lucide-react';
import { NavLink } from 'react-router-dom';
import { ThemeControl } from './ThemeControl';

const navigation = [
  { label: 'Dashboard', path: '/dashboard', icon: CircleGauge },
  { label: 'Tasks', path: '/tasks', icon: CheckSquare2 },
  { label: 'Planner', path: '/planner', icon: CalendarDays },
  { label: 'Habits', path: '/habits', icon: Leaf },
  { label: 'Goals', path: '/goals', icon: Goal },
  { label: 'Budget', path: '/budget', icon: Landmark },
  { label: 'Settings', path: '/settings', icon: Settings },
] as const;

export function Sidebar() {
  return (
    <aside className="sidebar">
      <div className="brand" aria-label="Zenfolio home">
        <div className="brand-mark" aria-hidden="true">
          Z
        </div>
        <div>
          <p className="brand-name">Zenfolio</p>
          <p className="brand-caption">Personal workspace</p>
        </div>
      </div>

      <nav aria-label="Primary navigation" className="sidebar-nav">
        {navigation.map(({ label, path, icon: Icon }) => (
          <NavLink
            key={path}
            to={path}
            className={({ isActive }) =>
              `nav-item${isActive ? ' nav-item-active' : ''}`
            }
          >
            <Icon aria-hidden="true" size={19} strokeWidth={1.8} />
            <span>{label}</span>
          </NavLink>
        ))}
      </nav>

      <div className="sidebar-footer">
        <ThemeControl />
        <p>Local and private</p>
      </div>
    </aside>
  );
}
