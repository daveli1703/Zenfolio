const days = [
  ['M', 1],
  ['T', 2],
  ['W', 4],
  ['T', 8],
  ['F', 16],
  ['S', 32],
  ['S', 64],
] as const;
export function WeekdaySelector({
  value,
  onChange,
}: {
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <fieldset className="weekday-picker">
      <legend>Schedule</legend>
      <div>
        {days.map(([label, bit], index) => (
          <label key={bit}>
            <input
              type="checkbox"
              aria-label={
                [
                  'Monday',
                  'Tuesday',
                  'Wednesday',
                  'Thursday',
                  'Friday',
                  'Saturday',
                  'Sunday',
                ][index]
              }
              checked={(value & bit) !== 0}
              onChange={() => onChange(value ^ bit)}
            />
            <span>{label}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}
