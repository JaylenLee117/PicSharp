import { Table } from '@tanstack/react-table';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { X as Close } from 'lucide-react';
import { DataTableFacetedFilter } from './data-table-faceted-filter';
import type { DataTableFacetedFilterProps } from './data-table-faceted-filter';
import { useEffect, useState } from 'react';

interface DataTableToolbarProps<TData> {
  table: Table<TData>;
  searchBy?: string;
  filters?: DataTableFacetedFilterProps<TData, any>[];
}

export function DataTableToolbar<TData>({
  table,
  searchBy,
  filters,
}: DataTableToolbarProps<TData>) {
  const isFiltered = table.getState().columnFilters.length > 0 || table.getState().globalFilter;

  return (
    <div className="flex items-center justify-between">
      <div className="flex flex-1 items-center space-x-2">
        {searchBy && (
          <SearchInput
            placeholder="Search ..."
            value={table.getState().globalFilter ?? ''}
            onChange={(value) => table.setGlobalFilter(value)}
            className="h-8 w-[150px] lg:w-[250px]"
          />
        )}
        {filters?.map((filter) => (
          <DataTableFacetedFilter<TData, any>
            id={filter.id}
            key={filter.id}
            column={table.getColumn(filter.id!)}
            title={filter.title}
            options={filter.options}
          />
        ))}
        {isFiltered && (
          <Button
            variant="ghost"
            onClick={() => {
              table.resetColumnFilters();
              table.resetGlobalFilter();
            }}
            className="h-8 px-2 lg:px-3"
          >
            Reset
            <Close className="ml-2 h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  );
}

function SearchInput({
  value: initialValue,
  onChange,
  debounceTime = 800,
  ...props
}: {
  value: string | number;
  onChange: (value: string | number) => void;
  debounceTime?: number;
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onChange'>) {
  const [value, setValue] = useState(initialValue);

  useEffect(() => {
    setValue(initialValue);
  }, [initialValue]);

  const handleKeyDown = (e: any) => {
    if (e.key === 'Enter') {
      onChange(value); // Invoke onChange with the current value
    }
  };

  const handleBlur = () => {
    onChange(value); // Invoke onChange with the current value
  };

  return (
    <Input
      {...props}
      value={value}
      onChange={(e) => setValue(e.target.value)}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
    />
  );
}
