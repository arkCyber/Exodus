/**
 * Exodus Browser — Print Client
 * Provides interface to print settings and operations
 */

import { invoke } from '@tauri-apps/api/core';

/** Print orientation */
export type PrintOrientation = 'portrait' | 'landscape';

/** Color mode */
export type ColorMode = 'color' | 'grayscale' | 'monochrome';

/** Paper size */
export type PaperSize = 'letter' | 'legal' | 'a4' | 'a3' | 'a5' | 'custom';

/** Margins */
export type Margins = {
  top: number;
  right: number;
  bottom: number;
  left: number;
};

/** Print settings */
export type PrintSettings = {
  default_printer?: string;
  orientation: PrintOrientation;
  paper_size: PaperSize;
  color_mode: ColorMode;
  margins: Margins;
  print_background: boolean;
  print_headers_footers: boolean;
  print_selection_only: boolean;
  copies: number;
  duplex: boolean;
  print_quality: number;
  scale: number;
  page_range?: string;
};

/** Print job */
export type PrintJob = {
  id: string;
  name: string;
  url: string;
  settings: PrintSettings;
  status: string;
  created_at: number;
  completed_at?: number;
};

/** Get print settings */
export async function getPrintSettings(): Promise<PrintSettings> {
  return invoke<PrintSettings>('get_print_settings');
}

/** Update print settings */
export async function updatePrintSettings(settings: PrintSettings): Promise<void> {
  return invoke<void>('update_print_settings', { settings });
}

/** Reset print settings to default */
export async function resetPrintSettings(): Promise<void> {
  return invoke<void>('reset_print_settings');
}

/** Get available printers */
export async function getAvailablePrinters(): Promise<string[]> {
  return invoke<string[]>('get_available_printers');
}

/** Get print history */
export async function getPrintHistory(): Promise<PrintJob[]> {
  return invoke<PrintJob[]>('get_print_history');
}

/** Clear print history */
export async function clearPrintHistory(): Promise<void> {
  return invoke<void>('clear_print_history');
}

/** Print to PDF */
export async function printToPdf(url: string, outputPath: string): Promise<void> {
  return invoke<void>('print_to_pdf', { url, outputPath });
}
