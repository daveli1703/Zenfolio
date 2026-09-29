import { HashRouter, Navigate, Route, Routes } from 'react-router-dom';
import { BudgetPage } from '../features/budget/BudgetPage';
import { DashboardPage } from '../features/dashboard/DashboardPage';
import { GoalsPage } from '../features/goals/GoalsPage';
import { HabitsPage } from '../features/habits/HabitsPage';
import { PlannerPage } from '../features/planner/PlannerPage';
import { SettingsPage } from '../features/settings/SettingsPage';
import { TasksPage } from '../features/tasks/TasksPage';
import { AppShell } from './AppShell';
import { ThemeProvider } from './ThemeProvider';

export function App() {
  return (
    <ThemeProvider>
      <HashRouter>
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<Navigate to="/dashboard" replace />} />
            <Route path="dashboard" element={<DashboardPage />} />
            <Route path="tasks" element={<TasksPage />} />
            <Route path="planner" element={<PlannerPage />} />
            <Route path="habits" element={<HabitsPage />} />
            <Route path="goals" element={<GoalsPage />} />
            <Route path="budget" element={<BudgetPage />} />
            <Route path="settings" element={<SettingsPage />} />
            <Route path="*" element={<Navigate to="/dashboard" replace />} />
          </Route>
        </Routes>
      </HashRouter>
    </ThemeProvider>
  );
}
