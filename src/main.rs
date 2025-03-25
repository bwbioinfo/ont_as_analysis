// Import all necessary types and traits from plotters
use plotters::prelude::*;
use plotters::style::full_palette::GREY_500;
// For error handling
use std::error::Error;
// For working with file paths
use std::path::Path;

/// A struct representing one row of the CSV input
#[derive(Debug)]
struct Record {
    time: f64,                     // Batch time (Unix timestamp, float)
    samples: f64,                  // Number of samples
    bases: f64,                    // Number of basecalls
    mean_qscore: f64,              // Average Q-score
    time_to_package_and_send: f64, // Time taken to package and send
    time_in_basecaller: f64,       // Time spent in basecalling
}

/// Reads the CSV file and parses it into a vector of `Record`s
fn parse_csv<P: AsRef<Path>>(csv_path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    // Open the CSV reader from the given file path
    let mut rdr = csv::Reader::from_path(csv_path)?;
    let mut data = Vec::new();

    // Iterate through each record (row) in the CSV
    for result in rdr.records() {
        let record = result?; // Handle CSV parsing errors

        // Parse relevant fields into f64 and construct a Record
        let r = Record {
            time: record.get(2).ok_or("Missing batch_time")?.parse()?,
            samples: record.get(3).ok_or("Missing samples")?.parse()?,
            bases: record.get(4).ok_or("Missing bases")?.parse()?,
            mean_qscore: record.get(6).ok_or("Missing mean_qscore")?.parse()?,
            time_to_package_and_send: record
                .get(7)
                .ok_or("Missing time_to_package_and_send")?
                .parse()?,
            time_in_basecaller: record.get(8).ok_or("Missing time_in_basecaller")?.parse()?,
        };

        // Push the parsed record into the data vector
        data.push(r);
    }

    // Sort records chronologically by time
    data.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

    Ok(data) // Return the parsed and sorted data
}

/// Plots a set of subplots showing different variables over time
fn plot_multi_series(data: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    // Create an SVG drawing area (1000px wide, 1200px tall)
    let root = BitMapBackend::new(output_path, (2200, 1800)).into_drawing_area();
    root.fill(&GREY_500)?; // Fill the background with white

    // Divide the root area into 5 stacked horizontal panels
    let split = root.split_evenly((5, 1));

    // List of fields to plot: (label, accessor function)
    // The accessor functions are boxed closures that extract a f64 value from a `Record`
    let fields: Vec<(&str, Box<dyn Fn(&Record) -> f64>)> = vec![
        ("Samples", Box::new(|r: &Record| r.samples)),
        ("Bases", Box::new(|r: &Record| r.bases)),
        ("Mean Q-score", Box::new(|r: &Record| r.mean_qscore)),
        (
            "Time to Package",
            Box::new(|r: &Record| r.time_to_package_and_send),
        ),
        (
            "Time in Basecaller",
            Box::new(|r: &Record| r.time_in_basecaller),
        ),
    ];

    // Iterate over each subplot panel and corresponding data field
    for (i, (title, accessor)) in fields.iter().enumerate() {
        let area = &split[i]; // Current subplot drawing area

        // Draw border around the subplot area
        let x_range = area.get_pixel_range().0.clone();
        let y_range = area.get_pixel_range().1.clone();

        let x0 = x_range.start;
        let x1 = x_range.end;
        let y0 = y_range.start;
        let y1 = y_range.end;

        area.draw(&Rectangle::new(
            [(x0, y0), (x1 - 1, y1 - 1)],
            BLACK.stroke_width(2),
        ))?;

        // Determine min/max time for x-axis bounds
        let _min_time = data.first().unwrap().time;
        let _max_time = data.last().unwrap().time;

        // Determine min/max time for x-axis bounds
        let min_time = data.first().unwrap().time;
        let max_time = data.last().unwrap().time;

        // Determine min/max value for y-axis bounds using the accessor
        let min_val = data
            .iter()
            .map(|r| accessor(r))
            .fold(f64::INFINITY, f64::min);
        let max_val = data
            .iter()
            .map(|r| accessor(r))
            .fold(f64::NEG_INFINITY, f64::max);

        // Create a chart for the current subplot
        // Draw border around the subplot area
        let mut chart = ChartBuilder::on(area)
            .caption(*title, ("sans-serif", 20)) // Title
            .margin(20) // Outer margin
            .x_label_area_size(50) // Space for x-axis labels
            .y_label_area_size(100) // Space for y-axis labels
            .build_cartesian_2d(min_time..max_time, min_val..max_val)?; // Axes ranges

        // Draw chart axes and grid
        chart
            .configure_mesh()
            .x_labels(5)
            .y_labels(5)
            // .disable_mesh() // Disable inner grid lines for cleaner look
            .x_desc("Batch Time")
            // Adjust label font size
            .x_label_style(("sans-serif", 20))
            .y_desc(*title)
            .draw()?;

        // Plot the data as a line series
        chart.draw_series(LineSeries::new(
            data.iter().map(|r| (r.time, accessor(r))),
            &GREEN, // Line color
        ))?;
    }

    Ok(())
}

use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    // Collect command-line arguments
    let args: Vec<String> = env::args().collect();

    // Expecting two arguments: the CSV path and output PNG path
    if args.len() != 3 {
        eprintln!("Usage: {} <input_csv> <output_png>", args[0]);
        std::process::exit(1);
    }

    let input_csv = &args[1];
    let output_png = &args[2];

    // Load and parse CSV data from file
    let data = parse_csv(input_csv)?;

    // Generate the subplot visualization and save to file
    plot_multi_series(&data, output_png)?;

    println!("Plot saved to {}", output_png);
    Ok(())
}
