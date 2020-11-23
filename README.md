# BouncingBalls

This experiment simulates N balls bouncing in a room for 10,000 iterations. There are two types of
simulation that can be run:

	(1)	N is dictated by the size of the room specified. 1 ball per one square unit is generated
	and given a random position and velocity. Upon collision with any other ball or surface of the
	room, the ball will change directions appropriately.

	(2)	This operates the same as the first type, with the added feature of adding or removing a
	component upon each collision.


To run the simulation, type at the command line

		~$ cargo run --size <integer> -- type <1/2>

In the <integer> field, enter any positive integer to represent the length of one side of the
square room being simulated. In the <1/2> field, enter the number for which type of experiment you
wish to simulate. 
