# Define the hosts, on which the tests will be run
hosts::Vector{String} = ["polkadot", "kagome"]

# Add locally built or downloaded adapters, testers, and hosts to PATH
ENV["PATH"] *= ":$(pwd())/bin"

# Tests directory
tests_dir::String = get(ENV ,"ZN_TESTS", "./tests")

# Test output path
tests_output::String = get(ENV, "ZN_TEST_OUTPUT", "./tests_output")

# Filter out the non-`.zndsl` files, just to be safe
tests::Vector{String} = filter(file -> endswith(file, ".zndsl"), readdir(tests_dir))

# Arrays to accumulate passed and failed test names
passed_tests::Vector{String} = []
failed_tests::Vector{String} = []

# Helper for piping both outputs
function redirect_stdout_stderr(dofunc, outfile, errfile)
    open(outfile, "w") do out
        open(errfile, "w") do err
            redirect_stdout(out) do
                redirect_stderr(err) do
                    dofunc()
                end
            end
        end
    end
end

# For each host ...
for host::String in hosts
    withenv("ZOMBIENET_DEFAULT_START_COMMAND" => host) do
        # ... run each test
        for test::String in tests
            if endswith(test, ".zndsl")
                # Extract the index and test name from the test path
                match_captures = match(r"(\d+)-(.*?)\.zndsl", basename(test)).captures
                index::Int64 = parse(Int64, match_captures[1])
                test_name::String = match_captures[2]

                # Prepare the `zombienet test` command
                command::Cmd = `node zombienet/javascript/packages/cli/dist/cli.js --provider native test $(joinpath(tests_dir, test))`
                println("Running test [$(host)] $(test_name)")

                # Try to run the test
                try
                    # Create the output files
                    output_stdout = joinpath(tests_output, "output_$(index)-$(test_name)_stdout.log")
                    output_stderr = joinpath(tests_output, "output_$(index)-$(test_name)_stderr.log")

                    # Try to run the command and capture its output
                    redirect_stdout_stderr(output_stdout, output_stderr) do
                        run(command)
                    end

                    # Add the test name to the passed tests array
                    push!(passed_tests, test_name)
                catch e
                    # Add the test name to the failed tests array
                    push!(failed_tests, test_name)
                end
            end
        end
    end
end

# Report the passed and failed tests
println("Passed tests:")
println(passed_tests)

println("Failed tests:")
println(failed_tests)
