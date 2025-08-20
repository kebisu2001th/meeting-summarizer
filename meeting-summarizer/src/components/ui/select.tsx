import { useState } from 'react';
import { ChevronDown, Check } from 'lucide-react';

interface Option {
  value: string;
  label: string;
  description?: string;
}

interface SelectProps {
  options: Option[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
}

export function Select({ options, value, onChange, placeholder = "Select an option", disabled = false }: SelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  
  const selectedOption = options.find(option => option.value === value);

  return (
    <div className="relative">
      <button
        type="button"
        className={`
          relative w-full cursor-default rounded-md border border-gray-300 bg-white py-2 pl-3 pr-10 text-left shadow-sm 
          focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500
          ${disabled ? 'cursor-not-allowed opacity-50' : 'hover:border-gray-400'}
        `}
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
      >
        <span className="block truncate">
          {selectedOption ? selectedOption.label : placeholder}
        </span>
        <span className="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
          <ChevronDown className="w-5 h-5 text-gray-400" />
        </span>
      </button>

      {isOpen && !disabled && (
        <>
          <div 
            className="fixed inset-0 z-10" 
            onClick={() => setIsOpen(false)}
          />
          <div className="absolute z-20 mt-1 w-full bg-white shadow-lg max-h-60 rounded-md py-1 text-base ring-1 ring-black ring-opacity-5 overflow-auto focus:outline-none">
            {options.map((option) => (
              <div
                key={option.value}
                className={`
                  cursor-pointer select-none relative py-2 pl-10 pr-4 hover:bg-blue-50
                  ${option.value === value ? 'bg-blue-50 text-blue-900' : 'text-gray-900'}
                `}
                onClick={() => {
                  onChange(option.value);
                  setIsOpen(false);
                }}
              >
                <div className="block">
                  <span className={`block truncate ${option.value === value ? 'font-medium' : 'font-normal'}`}>
                    {option.label}
                  </span>
                  {option.description && (
                    <span className="text-sm text-gray-500 mt-1 block">
                      {option.description}
                    </span>
                  )}
                </div>
                {option.value === value && (
                  <span className="absolute inset-y-0 left-0 flex items-center pl-3 text-blue-600">
                    <Check className="w-5 h-5" />
                  </span>
                )}
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
}