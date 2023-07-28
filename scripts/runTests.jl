# Define the hosts, on which the tests will be run
hosts::Vector{String} = ["polkadot", "kagome"]

# Add locally built or downloaded adapters, testers, and hosts to PATH
ENV["PATH"] *= ":$(pwd())/bin"

# Tests directory
tests_dir::String = get(ENV ,"ZN_TESTS", "./tests")

# Test output path
tests_output::String = get(ENV, "ZN_TEST_OUTPUT", "./tests_output")

# Arrays to accumulate passed and failed test names
passed_tests::Vector{String} = []
failed_tests::Vector{String} = []

# For each host ...
for host::String in hosts
    withenv("ZOMBIENET_DEFAULT_START_COMMAND" => host) do
        # Get host tests directory
        host_tests_dir::String = joinpath(tests_dir, host)
        # Filter out the non-`.zndsl` files for each host
        tests::Vector{String} = filter(file -> endswith(file, ".zndsl"), readdir(host_tests_dir))
        
        # ... run each test
        for test::String in tests
            if endswith(test, ".zndsl")
                # Extract the index and test name from the test path
                match_captures = match(r"(\d+)-(.*?)\.zndsl", basename(test)).captures
                index::Int64 = parse(Int64, match_captures[1])
                test_name::String = match_captures[2]

                # Prepare the `zombienet test` command
                command::Cmd = `zombienet-linux-x64 -p native test $(joinpath(host_tests_dir, test))`
                full_test_name::String = "[$(host)] $(test_name)"
                println("Running test $(full_test_name)")

                # Try to run the test
                try
                    # Try to run the command
                    run(command)

                    # Add the test name to the passed tests array
                    push!(passed_tests, full_test_name)
                catch e
                    showerror(stdout, e)
                    # Add the test name to the failed tests array
                    push!(failed_tests, full_test_name)
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

# Exit with an error code if any tests failed
if length(failed_tests) > 0
    exit(1)
end
