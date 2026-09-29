export function App() {
  return (
    <main className="mx-auto flex min-h-screen max-w-3xl items-center px-8 py-16">
      <section aria-labelledby="welcome-title" className="w-full">
        <div
          className="mb-8 flex h-12 w-12 items-center justify-center rounded-2xl bg-emerald-800 text-xl font-semibold text-white"
          aria-hidden="true"
        >
          P
        </div>
        <p className="mb-3 text-sm font-medium tracking-wide text-emerald-800">
          YOUR PERSONAL WORKSPACE
        </p>
        <h1
          id="welcome-title"
          className="text-4xl font-semibold tracking-tight text-slate-900"
        >
          A little more room for life.
        </h1>
        <p className="mt-5 max-w-xl text-lg leading-relaxed text-slate-600">
          One quiet place for your tasks, habits, goals, plans, and finances.
          Built for your desktop, with your data staying on your computer.
        </p>
        <div className="mt-10 rounded-2xl border border-slate-200 bg-white p-6">
          <h2 className="font-semibold text-slate-900">
            Personal Productivity
          </h2>
          <p className="mt-2 text-sm leading-relaxed text-slate-600">
            The foundation is in place. Your workspace and navigation are coming
            next.
          </p>
          <p className="mt-4 text-xs text-slate-500">
            No account required · No cloud services
          </p>
        </div>
      </section>
    </main>
  );
}
