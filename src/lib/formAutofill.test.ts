/**
 * Exodus Browser — formAutofill unit tests.
 */
import { describe, expect, it } from 'vitest';
import { fieldTypeKey, type FormAutofillEntry } from './formAutofill';

describe('formAutofill', () => {
  it('fieldTypeKey handles string field_type', () => {
    const entry = { field_type: 'email' } as FormAutofillEntry;
    expect(fieldTypeKey(entry)).toBe('email');
  });

  it('fieldTypeKey handles enum object field_type', () => {
    const entry = { field_type: { Email: null } } as unknown as FormAutofillEntry;
    expect(fieldTypeKey(entry)).toBe('email');
  });

  it('fieldTypeKey handles Custom variant', () => {
    const entry = { field_type: { Custom: 'company' } } as unknown as FormAutofillEntry;
    expect(fieldTypeKey(entry)).toBe('company');
  });
});
